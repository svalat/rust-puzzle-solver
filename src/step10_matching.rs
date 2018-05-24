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

//import
extern crate image;

//std
use std::f32;
use std::fs::File;
use std::io::Write;
use std::mem;

//extern
use image::{GrayImage,imageops,Luma};

//internal
use piece::{PieceFace,PieceVec,PieceMatch,Piece};
use std::cmp::Ordering;
use step5_corners;
use common;

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
		top: (face.bottom.0,-face.bottom.1),
		middle: (face.middle.0,-face.middle.1),
		bottom: (face.top.0,-face.top.1),
		mode: face.mode,
	}
}

fn rotate_point(point:(f32,f32),angle:f32) -> (f32,f32) {
	let (x,y) = point;
	let xf = x * angle.cos() - y * angle.sin();
	let yf = x * angle.sin() + y * angle.cos();
	(xf,yf)
}

fn rotate_point_center(point:(f32,f32),center:(f32,f32),center_after:(f32,f32),angle:f32) -> (f32,f32) {
	let (x,y) = point;
	let (cx,cy) = center;
	let (cx1,cy1) = center_after;
	let angle = angle.to_radians();
	let xf = cx1 + (x-cx) * angle.cos() - (y-cy) * angle.sin();
	let yf = cy1 + (x-cx) * angle.sin() + (y-cy) * angle.cos();
	(xf,yf)
}

fn rotate_face(face:&PieceFace,angle:f32) -> PieceFace {
	//compute
	let mut ret = PieceFace {
		top: rotate_point(face.top,angle),
		middle: rotate_point(face.middle,angle),
		bottom: rotate_point(face.bottom,angle),
		mode: face.mode,
	};

	//invert
	if ret.top.1 > ret.bottom.1 {
		mem::swap(&mut ret.top,&mut ret.bottom);
	}

	ret
}

fn check_quick_face_distance_mirrored(face1: &PieceFace,face2: &PieceFace) -> (f32,f32,PieceFace) {
	let face1 = face_mirror_on_y(face1);
	let face2 = face_mirror_on_y(face2);
	check_quick_face_distance(&face1,&face2)
}

fn check_quick_face_distance(face1: &PieceFace,face2: &PieceFace) -> (f32,f32,PieceFace) {
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
	//let face2 = rotate_face(&face2,angle);

	//calc distance
	let ret = calc_dist(face1.top,face2.top) + calc_dist(face1.middle,face2.middle) + calc_dist(face1.bottom,face2.bottom);

	(ret,angle,face2)
}

fn  cacl_rotate(face: usize, want_on: usize) -> usize {
	((want_on + 4) - face) % 4
}

fn get_rotated(img: &GrayImage,face: usize,want_on:usize) -> GrayImage {
	//calc roation
	let rot = cacl_rotate(face,want_on);

	//rotate
	let rotated;
	match rot {
		0 => rotated = img.clone(),
		1 => rotated = imageops::rotate90(img),
		2 => rotated = imageops::rotate180(img),
		3 => rotated = imageops::rotate270(img),
		_ => panic!("Invalid value should be 0,1,2 or 3 !"),
	}

	//ret
	rotated
}

fn rotate_face_center(piece_size: (u32,u32),face: &PieceFace, faceid: usize,want_on:usize) -> PieceFace {
	//calc roation
	let (w,h) = piece_size;
	let center = (w as f32/2.0,h as f32/2.0);
	let rot = cacl_rotate(faceid,want_on);
	let angle = rot as f32 * 90.0;

	//println!("BEFORE {} => {} => ({}) => {:?}",faceid,want_on,angle,face);

	//calculate after center
	let center2;
	match rot {
		0 => center2 = (0 as f32,0 as f32),
		1 => center2 = (h as f32,0 as f32),
		2 => center2 = (w as f32,h as f32),
		3 => center2 = (0 as f32,w as f32),
		_ => panic!("This should not append"),
	}

	//rotate
	let mut ret = PieceFace {
		top: rotate_point_center(face.top,(0.0,0.0),center2,angle),
		middle: rotate_point_center(face.middle,(0.0,0.0),center2,angle),
		bottom: rotate_point_center(face.bottom,(0.0,0.0),center2,angle),
		mode: face.mode,
	};

	//invert
	if ret.top.1 > ret.bottom.1 && (want_on == 1 || want_on == 3) {
		mem::swap(&mut ret.top,&mut ret.bottom);
	}

	//println!("AFTER => {:?}",ret);

	ret
}

fn add_mask(out: &mut GrayImage,mask: &GrayImage,pos: (u32,u32)) {
	let (w,h) = mask.dimensions();
	let (x0,y0) = pos;
	for y in 0..h {
		for x in 0..w {
			let color;
			{
				let m = mask.get_pixel(x,y);
				let o = out.get_pixel(x+x0,y+y0);
				color = Luma([m.data[0] + o.data[0]]);
			}
			out.put_pixel(x+x0,y+y0,color);
		}
	}
}

fn calc_right_mask_pos(left_face: &PieceFace,right_face: &PieceFace,left_pos:(u32,u32),offset:(i32,i32)) -> (u32,u32) {
	//calc angle
	//println!("FACE {:?} {:?}",left_face,right_face);
	let (x0,y0) = (left_pos.0 as f32,left_pos.1 as f32);
	let (ox,oy) = (offset.0 as f32,offset.1 as f32);
	let x = ox + x0 as f32 + left_face.top.0 - right_face.top.0;
	let y = oy + y0 as f32 + left_face.top.1 - right_face.top.1;
	//println!("MOVE {:?}",(x,y));
	(x as u32,y as u32)
}

fn count_superp(img: &GrayImage,rect:(u32,u32,u32,u32)) -> u32 {
	let (x0,y0,x1,y1) = rect;
	let c = Luma([common::MASK_PIECE_PIXEL * 2]);
	let back = Luma([common::MASK_BACKGROUND]);
	let mut cnt = 0;
	for y in x0..x1 {
		for x in y0..y1 {
			let color = img.get_pixel(x,y);
			if *color == c || *color == back {
				cnt += 1;
			}
		}
	}

	cnt
}

fn calc_face_mask_dist(left: &Piece, fid_left: usize,right: &Piece, fid_right: usize,id: u32,dump: i32) -> f32 {
	let mut min = f32::MAX;
	let base = format!("tmp-{}:{}-{}:{}.txt",left.id,fid_left,right.id,fid_right);
    let mut file = File::create(base).unwrap();
	for y in -common::MATCH_MASK_OFFET..common::MATCH_MASK_OFFET {
		for x in -common::MATCH_MASK_OFFET..common::MATCH_MASK_OFFET {
			let dist = calc_face_mask_dist_offset(left,fid_left,right,fid_right,id,dump,(x*2,y*2));
			file.write_fmt(format_args!("{} {} {}\n",x,y,dist));
			if dist < min {
				min = dist;
			}
		}
	}

	min
}

fn move_rect(rect:(u32,u32,u32,u32),pos:(u32,u32)) -> (u32,u32,u32,u32) {
	(rect.0 + pos.0,rect.1 + pos.1, rect.2 + pos.0, rect.3 + pos.1)
}

fn get_intersection(left: &GrayImage,pleft:(u32,u32),right: &GrayImage,pright:(u32,u32)) -> (u32,u32,u32,u32) {
	//extract surrounding
	let rect_left = step5_corners::extract_surrounding_rect(left);
	let rect_right = step5_corners::extract_surrounding_rect(right);

	//move
	let rect_left = move_rect(rect_left,pleft);
	let rect_right = move_rect(rect_right,pright);

	//intersect
	let xmin = rect_right.0;
	let xmax = rect_left.2;
	let ymin = rect_left.1.max(rect_right.1);
	let ymax = rect_left.3.min(rect_right.3);

	(xmin,ymin,xmax,ymax)
}

fn calc_face_mask_dist_offset(left: &Piece, fid_left: usize,right: &Piece, fid_right: usize,id: u32,dump: i32,offset:(i32,i32)) -> f32 {
	//compute size
	let (lw,lh) = left.mask.dimensions();
	let (rw,rh) = right.mask.dimensions();
	let size = lw.max(lh).max(rw).max(rh) * 2;

	//build out image & roate masks
	let mut img = GrayImage::new(size,size);
	let left_mask = get_rotated(&left.mask,fid_left,1);
	let right_mask = get_rotated(&right.mask,fid_right,3);

	//prepare points
	let left_face = rotate_face_center((lw,lh),&left.faces[fid_left],fid_left,1);
	let right_face = rotate_face_center((rw,rh),&right.faces[fid_right],fid_right,3);

	//fill unintesting pixels
	//TODO

	//rotate right
	//let right_mask = imageops::rotate180(&right_mask);
	let right_face = rotate_face_center((rw,rh),&right.faces[fid_right],fid_right,3);

	//calculate piece position
	let (lw,lh) = left_mask.dimensions();
	let left_pos = (size/2 - lw,size/2-lh/2);
	//println!("LEFT {:?}",left_pos);
	let right_pos = (size/2, size/2);
	let right_pos = calc_right_mask_pos(&left_face,&right_face,left_pos,offset);

	//draw
	add_mask(&mut img,&left_mask,left_pos);
	//step5_corners::draw_point(&mut img,(left_pos.0+left_face.top.0 as u32,left_pos.1+left_face.top.1 as u32));
	//step5_corners::draw_point(&mut img,(left_pos.0+left_face.bottom.0 as u32,left_pos.1+left_face.bottom.1 as u32));
	//step5_corners::draw_point(&mut img,(left_pos.0+left_face.middle.0 as u32,left_pos.1+left_face.middle.1 as u32));
	
	add_mask(&mut img,&right_mask,right_pos);
	//step5_corners::draw_point(&mut img,(right_pos.0+right_face.top.0 as u32,right_pos.1+right_face.top.1 as u32));
	//step5_corners::draw_point(&mut img,(right_pos.0+right_face.middle.0 as u32,right_pos.1+right_face.middle.1 as u32));
	//step5_corners::draw_point(&mut img,(right_pos.0+right_face.bottom.0 as u32,right_pos.1+right_face.bottom.1 as u32));

	//save into file
	if dump == -10 && (left.id == 3 && fid_left == 1 || right.id == 3 && fid_right == 1) {
		let fname = format!("step-10-mask-match-{:05}-{}:{}-{}:{}-{}:{}.png",id,left.id,fid_left,right.id,fid_right,offset.0,offset.1);
		img.save(fname).unwrap();
	}

	//ret
	let rect = get_intersection(&left_mask,left_pos,&right_mask,right_pos);
	//println!("interect {:?}",rect);
	count_superp(&img,rect) as f32
}

pub fn compute_matching(pieces: &mut PieceVec, dump:i32) {
	//to extract media dist
	let mut full_soluce: Vec<(f32,f32,bool,usize,usize,usize,usize)> = vec!();
	let mut file: Option<File> = None;
	let mut file2: Option<File> = None;

	//open for dump
	//dump db into file
    if dump == 0 || dump == 10 {
        let base = format!("step-10-matching.txt");
        file = Some(File::create(base).unwrap());
		let base2 = format!("step-10-matching-2.txt");
        file2 = Some(File::create(base2).unwrap());
    }

	//loop on all pieces
	for i1 in 0..pieces.len() {
		let p1 = &pieces[i1].lock().unwrap();
		
		//match with all others
		for i2 in (i1+1)..pieces.len() {
			let p2 = &pieces[i2].lock().unwrap();

			//loop on all faces to match
			for fid1 in 0..4 {
				let face1 = &p1.faces[fid1];
				let face1 = rotate_face_center(p1.mask.dimensions(),&face1,fid1,1);
				let face1 = move_face(&face1,face1.top.0,face1.top.1);

				for fid2 in 0..4 {
					let face2 = &p2.faces[fid2];
					let face2 = rotate_face_center(p2.mask.dimensions(),&face2,fid2,3);

					//check if can match then match
					if face1.mode != face2.mode {
						let (dist1,angle1,f1) = check_quick_face_distance(&face1,&face2);
						let (dist2,angle2,f2) = check_quick_face_distance_mirrored(&face1,&face2);
						let dist = dist1.min(dist2);
						if dist1 < dist2 {
							full_soluce.push((dist,angle1,false,i1,fid1,i2,fid2));;
							match file.as_mut() {
								Some(f) => f.write_fmt(format_args!("Match {}:{} <-> {}:{} -> {} -> {} -> {:?} -- {:?} --> {:?}\n",i1,fid1,i2,fid2,dist,angle1,face1,face2,f1)).unwrap(),
								None => {}
							}
						} else {
							full_soluce.push((dist,angle2,true,i1,fid1,i2,fid2));
							match file.as_mut() {
								Some(f) => f.write_fmt(format_args!("Match {}:{} <-> {}:{} -> {} -> {} -> {:?} -- {:?} --> {:?}\n",i1,fid1,i2,fid2,dist,angle2,face1,face2,f2)).unwrap(),
								None => {}
							}
						}
					}
				}
			}
		}
	}

	//median
	println!("Calc median");
	full_soluce.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    let mid = full_soluce.len() / 2;
	let cut = full_soluce[mid].0;///2.0;
	println!("median = {}, median/2 = {}",full_soluce[mid].0,cut);

	//apply second step filter
	let mut filtered_soluce: Vec<(f32,f32,bool,usize,usize,usize,usize)> = vec!();
	let mut id = 0;
	for m in full_soluce {
		let (dist,angle,_mirrored,id1,fid1,id2,fid2) = m;
		if dist < cut {
			let p1 = &pieces[id1].lock().unwrap();
			let p2 = &pieces[id2].lock().unwrap();
			let fdist = calc_face_mask_dist(p1,fid1,p2,fid2,id,dump);
			match file2.as_mut() {
				Some(f) => f.write_fmt(format_args!("Match {}:{} <-> {}:{} -> {}\n",id1,fid1,id2,fid2,fdist)).unwrap(),
				None => {}
			}
			filtered_soluce.push((fdist,angle,_mirrored,id1,fid1,id2,fid2));
			id += 1;
		}
	}

	//apply cut on new filtered list
	println!("Calc median");
	filtered_soluce.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    let mid = filtered_soluce.len() / 2;
	let cut = filtered_soluce[mid].0/2.0;
	println!("median = {}, median/2 = {}",filtered_soluce[mid].0,cut);

	//loop and save
	for m in filtered_soluce {
		let (dist,angle,_mirrored,id1,fid1,id2,fid2) = m;
		if dist <= cut {
			{
				let p1 = &mut pieces[id1].lock().unwrap();
				p1.matches[fid1].push(PieceMatch{
					piece: id2,
					side: fid2,
					angle: angle,
					distance: dist,
				});
			}

			{
				let p2 = &mut pieces[id2].lock().unwrap();
				p2.matches[fid2].push(PieceMatch{
					piece: id1,
					side: fid1,
					angle: angle,
					distance: dist,
				});
			}
		}
	}
}