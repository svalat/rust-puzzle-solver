/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// Now we know more of less the macthing between pieces we can try to brut force to build
/// a soluction following the matching and try to match most of the given pieces.

//load external
extern crate image;

//std
use std::usize;
use std::ops::Add;
use std::fmt;

//extern
use ndarray::{Array2,Axis};
use num_traits::identities::Zero;

//local
use piece::{PieceVec,TOP,RIGHT,LEFT,BOTTOM,PieceMatchVec};

//consts
pub const NONE: usize = usize::MAX;

//element
#[derive(Debug,Clone,PartialEq)]
pub struct SoluceElmt {
    pub piece_id: usize,
    pub rotation: usize,
}

impl fmt::Display for SoluceElmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The `f` value implements the `Write` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
		if self.piece_id == NONE {
			write!(f, "( ,  )")
		} else {
        	write!(f, "({}, {})", self.piece_id, self.rotation)
		}
    }
}

impl Add for SoluceElmt {
    type Output = Self;

    fn add(self, _other:Self) -> Self {
        panic!("Should not be called");
    }
}


impl Default for SoluceElmt {
    fn default() -> Self {
        Self {
            piece_id: NONE,
            rotation: 0,
        }
    }
}

impl Zero for SoluceElmt {
    fn zero() -> Self {
        Self::default()
    }

    fn is_zero(&self) -> bool {
        let mut ret = false;
        if self.piece_id == NONE && self.rotation == 0 {
            ret = true;
        }
        ret
    }
}

//define a soluce
pub type Soluce = Array2<SoluceElmt>;
type PieceUsage = Vec<bool>;
type SoluceVec = Vec<Soluce>;

//to track final solutions
struct SoluceProposal {
    list: SoluceVec,
    nb: u32,
}

fn cell_has_piece(current: &Soluce,x : usize, y: usize, dx: i32, dy: i32) -> Option<(usize,usize)> {
    let x = x as i32 + dx;
    let y = y as i32 + dy;
    let (w,h) = (current.len_of(Axis(0)),current.len_of(Axis(1)));
    let (w,h) = (w as i32,h as i32);
    let mut ret = false;
    if x >= 0 && x < w && y >= 0 && y < h {
        let cell = current.get((x as usize,y as usize)).unwrap();
        if cell.piece_id != NONE {
            //println!("Found {} in {:?}",cell.piece_id,(x,y));
            ret = true;
        }
    }

    if ret {
        Some((x as usize,y as usize))
    } else {
        None
    }
}

fn has_neighboor(current: &Soluce,x : usize, y: usize) -> Option<(usize,usize)> {
    let mut ret: Option<(usize,usize)> = None;
    for dx in -1i32..2i32 {
        for dy in -1i32..2i32 {
            if (dx == 0 || dy == 0) && dx != dy {
                let status = cell_has_piece(current,x,y,dx,dy);
                match status {
                    Some(coord) => {
                        ret = Some(coord);
                        //println!("Has neighboor : {:?} -> {:?}",(x,y),(dx,dy));
                    },
                    None => {},
                }
            }
        }
    } 

    //no neighboor
    ret
}

fn extract_candidates(current: & Soluce,pos: (usize,usize),exist:(usize,usize)) -> (usize,usize) {
    //vars
    let (x,y) = pos;
    let (xn,yn) = exist;
    let neighboor = current.get(exist).unwrap();
        
    //which side we want
    //eg if neighboot is on left we want is right face
    let side: usize;
    if xn < x {
        side = RIGHT;
    } else if xn > x {
        side = LEFT;
    } else if yn < y {
        side = BOTTOM;
    } else if yn > y {
        side = TOP;
    } else {
        panic!("This should not append !");
    }

    //apply rotation
    let side = (side + 4 - neighboor.rotation) % 4;

    //ret
    (neighboor.piece_id,side)
}

fn calc_rotation(pos:(usize,usize),neighboor:(usize,usize),curside:usize) -> usize {
	//vars
	let (x,y) = pos;
    let (xn,yn) = neighboor;

	//which side we want
    //eg if neighboor is on left we want to put the new piece face on left
    let wanted: usize;
    if xn < x {
        wanted = LEFT;
    } else if xn > x {
        wanted = RIGHT;
    } else if yn < y {
        wanted = TOP;
    } else if yn > y {
        wanted = BOTTOM;
    } else {
        panic!("This should not append !");
    }

	//compute rotation
	let rot:usize;
	if curside > wanted {
		rot = wanted + 4 - curside;
	} else {
		rot = wanted - curside;
	}

	rot
}

fn check_match_one_neighboot(pieces: &PieceVec, current: &mut Soluce, pos: (usize,usize),delta:(i32,i32),side:usize) -> bool {
	let (x,y) = pos;
	let (dx,dy) = delta;
	let x = x as i32 + dx;
    let y = y as i32 + dy;
    let (w,h) = (current.len_of(Axis(0)),current.len_of(Axis(1)));
    let (w,h) = (w as i32,h as i32);
    let mut ret = false;
    if x >= 0 && x < w && y >= 0 && y < h {
        let neighboor = current.get((x as usize,y as usize)).unwrap();
		let cur = current.get(pos).unwrap();
        if neighboor.piece_id != NONE {
            //calc real face the neighboor piece expose to current one
			let nfid = ((side + 2) + 4 - neighboor.rotation) % 4;

			//calc real face the current piece export to neighboor
			let fid = (side + 4 - cur.rotation) % 4;

			//search in list
			{
				let p = pieces[cur.piece_id].lock().unwrap();
				for link in p.matches[fid].iter() {
					if link.piece == neighboor.piece_id && link.side == nfid {
						ret = true;
					}
				}
			}
        } else {
			ret = true
		}
    } else {
		ret = true
	}

    ret
}

fn check_match_all_neighboors(pieces: &PieceVec, current: &mut Soluce, pos: (usize,usize)) -> bool {
	check_match_one_neighboot(pieces,current,pos,(-1,0),LEFT)
	&& check_match_one_neighboot(pieces,current,pos,(1,0),RIGHT)
	&& check_match_one_neighboot(pieces,current,pos,(0,-1),TOP)
	&& check_match_one_neighboot(pieces,current,pos,(0,1),BOTTOM)
}

fn find_fist_non_empty(soluce:&Soluce) -> (usize,usize) {
	let (w,h) = (soluce.len_of(Axis(0)),soluce.len_of(Axis(1)));
	let mut ret = (0,0);
	
	for y in 0..h {
		for x in 0..w {
			let local = soluce.get((x,y)).unwrap();
			if local.piece_id != NONE && ret == (0,0) {
				ret = (x,y)
			}
		}
	}

	ret
}

fn check_if_same_soluce(s1: &Soluce,s2:&Soluce) -> bool {
	//infos
	let (w,h) = (s1.len_of(Axis(0)),s1.len_of(Axis(1)));
	let mut ret = true;

	//extract first non empty
	let (x0,y0) = find_fist_non_empty(s1);
	let (x1,y1) = find_fist_non_empty(s1);
	let (ww,hh) = (w - x0.max(x1),h - y0.max(y1));

	//loop
	for y in 0..hh {
		for x in 0..ww {
			let p1 = s1.get((x+x0,y+y0)).unwrap();
			let p2 = s2.get((x+x1,y+y1)).unwrap();
			if p1 != p2 {
				ret = false;
			}
		}
	}

	ret
}

fn search_next_step_recurse(pieces: &PieceVec, current: &mut Soluce, usage: &mut PieceUsage, proposal: &mut SoluceProposal,depth:u32,dist:f32) {
    //search an intersting position
    let (w,h) = (current.len_of(Axis(0)),current.len_of(Axis(1)));
	let mut found = false;

    //search a position which has neighboors
    for y in 0..h {
        for x in 0..w {
			let empty;
			{
				let local = current.get((x,y)).unwrap();
				empty = local.piece_id == NONE;
			}

			if empty {
				//check if has neighboor pices
				let status = has_neighboor(current,x,y);
				match status {
					Some(coord) => {
						//extract candidate for local position
						let (id,nside) = extract_candidates(current,(x,y),coord);
						let candidates: PieceMatchVec;
						{
							let n = pieces[id].lock().unwrap();
							//println!("=======> {:?} <-> {:?} ==> {} : {}",(x,y),coord,id,nside);
							//println!("Candidated:0 {:?}",n.matches[0]);
							//println!("Candidated:1 {:?}",n.matches[1]);
							//println!("Candidated:2 {:?}",n.matches[2]);
							//println!("Candidated:3 {:?}",n.matches[3]);
							candidates = n.matches[nside].clone();
						}

						//loop on candidates
						for c in candidates.iter() {
							//println!("TTry {}",c.piece);
							//check if already in use
							if !usage[c.piece] {
								//calc rotation to place the piece
								let rot = calc_rotation((x,y),coord,c.side);
								//println!("Try {}, {}",c.piece,rot);

								//setup piece in place
								{
									//println!("Put {}:{} in {:?}",c.piece,rot,(x,y));
									let mut cell = current.get_mut((x,y)).unwrap();
									cell.piece_id = c.piece;
									cell.rotation = rot;
									usage[c.piece] = true;
								}

								//check if match with all neighboors
								if check_match_all_neighboors(pieces,current,(x,y)) {
									//println!("Recurse {}",depth);
									search_next_step_recurse(pieces,current,usage,proposal,depth+1,dist + c.distance);
									found = true;
								}

								//undo to try another thing
								{
									let mut cell = current.get_mut((x,y)).unwrap();
									cell.piece_id = NONE;
									cell.rotation = 0;
									usage[c.piece] = false;
								}
							}
						}
					},
					None => {}
				}
			}
		}
    }

	//if not found any piece to place, end recurse
	//so we check if we have more piece in place than current solution and save the solution
	if !found {
		//count pieces used
		let mut cnt = 0;
		for v in usage {
			if *v {
				cnt+=1;
			}
		}

		//println!("Ok solution : {} {}",cnt,dist);
		if cnt > proposal.nb {
			println!("CLEAR");
			proposal.list.clear();
			proposal.nb = cnt;
		}
		if cnt == proposal.nb {
			println!("KEEP -> {}",proposal.list.len());
			let mut keep = true;

			//compare with existing sol
			for s in proposal.list.iter() {
				if check_if_same_soluce(s,current) {
					keep = false;
				}
			}

			//filter
			if keep {
				let copy = current.clone();
				proposal.list.push(copy);
				println!("{}",current);
			}

			//cut
			if proposal.list.len() >= 400 {
				panic!("Too many solution, piece matching was not restrictive enougth....");
			}
		}
	}
}

pub fn build_solution(pieces: &PieceVec, _dump:i32) -> Soluce {
    //estimate size & middle pos
    let size = pieces.len() * 2;
    let (x,y) = (size / 2, size / 2);

    //init a solution
    let mut current = Soluce::zeros((size,size));
    let mut usage:PieceUsage = vec!();
    let mut proposal = SoluceProposal {
        list:vec!(),
        nb:0,
    };

    //init
    for _ in 0..pieces.len() {
        usage.push(false);
    }

    //loop on all piece to start
    for i in 0..1 { //pieces.len() {
        //place to center
        {
            let mut cell = current.get_mut((x,y)).unwrap();
			assert!(cell.piece_id == NONE);
            cell.piece_id = i;
            cell.rotation = 0;
        }

        //mark as used
        usage[i] = true;

        //serch next step
        search_next_step_recurse(pieces,&mut current,&mut usage,&mut proposal,1,0.0);

        //remove piece for next round
        usage[i] = false;

		//place to center
        {
            let mut cell = current.get_mut((x,y)).unwrap();
			assert!(cell.piece_id == i);
            cell.piece_id = NONE;
            cell.rotation = 0;
        }
    }

	//print solutions
	println!("Found {} solutions with {} pieces",proposal.list.len(),proposal.nb);

	//print first solution
	for sol in proposal.list.iter() {
		println!("{}",sol);
	}	

	//return for test
	proposal.list[0].clone()
}

#[cfg(test)]
mod test {
	use std::sync::{Arc,Mutex};
	use image::{RgbaImage};
	use step11_build::*;
	use piece::{PieceVec,PieceMatch,Piece};

	#[test]
	fn builder_solve_exact() {
		let image = RgbaImage::new(100,100);
		let rect = (0,0,10,10);
		let back = image.get_pixel(0,0);
		let mut all: PieceVec = Vec::new();
		for i in 0..9 {
			all.push(Arc::new(Mutex::new(Piece::new(&image,&back,rect,i))));
		}

		//let links
		{
			let mut p0 = all[0].lock().unwrap();
			let mut p1 = all[1].lock().unwrap();
			let mut p2 = all[2].lock().unwrap();
			let mut p3 = all[3].lock().unwrap();
			let mut p4 = all[4].lock().unwrap();
			let mut p5 = all[5].lock().unwrap();
			let mut p6 = all[6].lock().unwrap();
			let mut p7 = all[7].lock().unwrap();
			let mut p8 = all[8].lock().unwrap();

			p5.matches[0].push(PieceMatch{piece:7,side:3,angle:0.0,distance:0.0});
			p7.matches[0].push(PieceMatch{piece:3,side:2,angle:0.0,distance:0.0});
			p4.matches[1].push(PieceMatch{piece:8,side:0,angle:0.0,distance:0.0});
			p8.matches[1].push(PieceMatch{piece:6,side:3,angle:0.0,distance:0.0});
			p7.matches[1].push(PieceMatch{piece:4,side:2,angle:0.0,distance:0.0});
			p3.matches[1].push(PieceMatch{piece:2,side:2,angle:0.0,distance:0.0});
			p4.matches[3].push(PieceMatch{piece:2,side:1,angle:0.0,distance:0.0});
			p2.matches[3].push(PieceMatch{piece:0,side:2,angle:0.0,distance:0.0});
			p2.matches[0].push(PieceMatch{piece:1,side:2,angle:0.0,distance:0.0});

			p7.matches[3].push(PieceMatch{piece:5,side:0,angle:0.0,distance:0.0});
			p3.matches[2].push(PieceMatch{piece:7,side:0,angle:0.0,distance:0.0});
			p8.matches[0].push(PieceMatch{piece:4,side:1,angle:0.0,distance:0.0});
			p6.matches[3].push(PieceMatch{piece:8,side:1,angle:0.0,distance:0.0});
			p4.matches[2].push(PieceMatch{piece:7,side:1,angle:0.0,distance:0.0});
			p2.matches[2].push(PieceMatch{piece:3,side:1,angle:0.0,distance:0.0});
			p2.matches[1].push(PieceMatch{piece:4,side:3,angle:0.0,distance:0.0});
			p0.matches[2].push(PieceMatch{piece:2,side:3,angle:0.0,distance:0.0});
			p1.matches[2].push(PieceMatch{piece:2,side:0,angle:0.0,distance:0.0});
		}

		//build
		let sol = build_solution(&all,-1);

		//check solution
		assert_eq!(*sol.get((9,9)).unwrap(), SoluceElmt{piece_id:0,rotation:0});
		assert_eq!(*sol.get((10,10)).unwrap(), SoluceElmt{piece_id:1,rotation:1});
		assert_eq!(*sol.get((9,10)).unwrap(), SoluceElmt{piece_id:2,rotation:1});
		assert_eq!(*sol.get((8,10)).unwrap(), SoluceElmt{piece_id:3,rotation:0});
		assert_eq!(*sol.get((9,11)).unwrap(), SoluceElmt{piece_id:4,rotation:1});
		assert_eq!(*sol.get((7,11)).unwrap(), SoluceElmt{piece_id:5,rotation:1});
		assert_eq!(*sol.get((10,12)).unwrap(), SoluceElmt{piece_id:6,rotation:0});
		assert_eq!(*sol.get((8,11)).unwrap(), SoluceElmt{piece_id:7,rotation:0});
		assert_eq!(*sol.get((9,12)).unwrap(), SoluceElmt{piece_id:8,rotation:0});
	}
}
