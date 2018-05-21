/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// Now we know more of less the macthing between pieces we can try to brut force to build
/// a soluction following the matching and try to match most of the given pieces.

//std
use std::usize;
use std::ops::Add;

//extern
use ndarray::{Array2,Axis};
use num_traits::identities::Zero;

//local
use piece::PieceVec;

//consts
const NONE: usize = usize::MAX;

//element
#[derive(Clone)]
struct SoluceElmt {
    piece_id: usize,
    rotation: usize,
}

impl Add for SoluceElmt {
    type Output = Self;

    fn add(self, other:Self) -> Self {
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
type Soluce = Array2<SoluceElmt>;
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
    for dx in -1i32..1i32 {
        for dy in -1i32..1i32 {
            let status = cell_has_piece(current,x,y,dx,dy);
            match status {
                Some(coord) => ret = Some(coord),
                None => {},
            }
        }
    } 

    //no neighboor
    ret
}

fn search_next_step_recurse(pieces: &PieceVec, current: &mut Soluce, usage: &mut PieceUsage, proposal: &mut SoluceProposal,depth:u32) {
    //search an intersting position
    let (w,h) = (current.len_of(Axis(0)),current.len_of(Axis(1)));

    //search a position which has neighboors
    for y in 0..h {
        for x in 0..w {
            let status = has_neighboor(current,x,y);
            match status {
                Some(coord) => {
                    
                },
                None => {}
            }
        }
    }
}

pub fn build_solution(pieces: &PieceVec, dump:i32) {
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
    for i in 0..size {
        usage.push(false);
    }

    //loop on all piece to start
    for i in 0..size {
        //place to center
        {
            let mut cell = current.get_mut((x,y)).unwrap();
            cell.piece_id = i;
            cell.rotation = 0;
        }

        //mark as used
        usage[i] = true;

        //serch next step
        search_next_step_recurse(pieces,&mut current,&mut usage,&mut proposal,1);

        //remove piece for next round
        usage[i] = false;
    }
}
