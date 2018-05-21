/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// This module implement the maching detection between faces, first by using only the
/// particular points (corner, bump, holes) extracted by previous steps, them
/// moving to a more in depth analysis with mask pixel maching.

//std
use std::f32;

//internal
use piece::{PieceFace,PieceVec};
use std::cmp::Ordering;

fn move_face(face: &PieceFace,dx:f32,dy:f32) -> PieceFace {
	PieceFace {
		top: (face.top.0 - dx,face.top.1 - dy),
		middle: (face.middle.0 - dx,face.middle.1 - dy),
		bottom: (face.bottom.0 - dx,face.bottom.1 - dy),
		mode: face.mode,
	}
}

fn calc_dist(p1:(f32,f32),p2:(f32,f32)) -> f32 {
	let (x1,y1) = p1;
	let (x2,y2) = p2;
	let dx = x2-x1;
	let dy = y2-y1;
	let ret = (dx*dx + dy*dy).sqrt();
	ret
}

fn face_mirror_on_x(face: &PieceFace) -> PieceFace {
	PieceFace {
		top: (-face.top.0,face.top.1),
		middle: (-face.middle.0,face.middle.1),
		bottom: (-face.bottom.0,face.bottom.1),
		mode: face.mode,
	}
}

fn face_mirror_on_y(face: &PieceFace) -> PieceFace {
	PieceFace {
		top: (face.top.0,-face.top.1),
		middle: (face.middle.0,-face.middle.1),
		bottom: (face.bottom.0,-face.bottom.1),
		mode: face.mode,
	}
}

fn rotate_point(point:(f32,f32),angle:f32) -> (f32,f32) {
	let (x,y) = point;
	let x = x * angle.cos() - y * angle.sin();
	let y = x * angle.sin() + y * angle.cos();
	(x,y)
}

fn rotate_face(face:&PieceFace,angle:f32) -> PieceFace {
	PieceFace 	{
		top: rotate_point(face.top,angle),
		middle: rotate_point(face.middle,angle),
		bottom: rotate_point(face.bottom,angle),
		mode: face.mode,
	}
}

fn check_quick_face_distance_mirrored(face1: &PieceFace,face2: &PieceFace) -> (f32,f32,PieceFace) {
	let face1 = face_mirror_on_y(face1);
	let face2 = face_mirror_on_y(face2);
	check_quick_face_distance(&face1,&face2)
}

fn check_quick_face_distance(face1: &PieceFace,face2: &PieceFace) -> (f32,f32,PieceFace) {
	//mirror
	let face2 = face_mirror_on_x(face2);

	//calcule offset to match top
	let dx = face2.top.0 - face1.top.0;
	let dy = face2.top.1 - face1.top.1;

	//applt offset
	let face2 = move_face(&face2,dx,dy);

	//calc angle
	let (x0,y0) = face1.bottom;
	let (x1,y1) = face2.bottom;
	let (dx,dy) = (x1-x0,y1-y0);
	let a = (x0*x0+y0*y0).sqrt();
	let b = (x1*x1+y1*y1).sqrt();
	let c = (dx*dx+dy*dy).sqrt();
	let cos = (a*a+b*b-c*c)/(2.0*a*b);
	let mut angle = cos.acos();
	if x0 < x1 {
		angle = -angle;
	}

	//rotate face2
	let face2 = rotate_face(&face2,angle);

	//calc distance
	let ret = calc_dist(face1.top,face2.top) + calc_dist(face1.middle,face2.middle) + calc_dist(face1.bottom,face2.bottom);

	(ret,angle,face2)
}

pub fn compute_matching(pieces: &PieceVec) {
	//to extract media dist
	let mut all: Vec<f32> = vec!();

	//loop on all pieces
	for i1 in 0..pieces.len() {
		let p1 = &pieces[i1].lock().unwrap();
		
		//match with all others
		for i2 in (i1+1)..pieces.len() {
			let p2 = &pieces[i2].lock().unwrap();

			//loop on all faces to match
			for fid1 in 0..4 {
				let face1 = &p1.faces[fid1];
				let face1 = move_face(&face1,face1.top.0,face1.top.1);

				for fid2 in 0..4 {
					let face2 = &p2.faces[fid2];

					//check if can match then match
					if face1.mode != face2.mode {
						let (dist1,angle1,f1) = check_quick_face_distance(&face1,face2);
						let (dist2,angle2,f2) = check_quick_face_distance_mirrored(&face1,&face2);
						let dist = dist1.min(dist2);
						all.push(dist);
						if dist1 < dist2 {
							println!("Match {}:{} <-> {}:{} -> {} -> {} -> {:?} -- {:?} --> {:?}",i1,fid1,i2,fid2,dist,angle1,face1,face2,f1);
						} else {
							println!("Match {}:{} <-> {}:{} -> {} -> {} -> {:?} -- {:?} --> {:?}",i1,fid1,i2,fid2,dist,angle2,face1,face2,f2);
						}
					}
				}
			}
		}
	}

	//median
	println!("Calc median");
	all.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let mid = all.len() / 2;
	println!("median = {}, median/2 = {}",all[mid],all[mid/2]);
}