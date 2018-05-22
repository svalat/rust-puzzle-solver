/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// After building the solution we want to draw it into a picture to easy check it

//load external
extern crate image;
extern crate imageproc;

//external
use image::{RgbaImage,Luma,imageops};
use ndarray::Axis;
use imageproc::{drawing,rect};
use std::usize;

//internal
use step11_build::{Soluce,NONE};
use step5_corners::extract_surrounding_rect;
use piece::{Piece,PieceVec};
use common;

type SurroundVec = Vec<(u32,u32,u32,u32)>;

fn build_surround(pieces: &PieceVec) -> SurroundVec {
	let mut ret: SurroundVec = vec!();

	for p in pieces.iter() {
		//extract rect
		let (xmin,ymin,xmax,ymax) = extract_surrounding_rect(&p.lock().unwrap().mask);

		//push
		ret.push((xmin, ymin, xmax - xmin, ymax - ymin));
	}

	ret
}

pub fn max_size(surround: &SurroundVec) -> (u32,u32) {
	let mut wmax = 0;
	let mut hmax = 0;

	for s in surround.iter() {
		let (_,_,w,h) = s;
		if *w > wmax {
			wmax = *w;
		}
		if *h > hmax {
			hmax = *h;
		}
	}

	let m = wmax.max(hmax);
	(m,m)
}

fn get_soluce_size(soluce:&Soluce) -> (usize,usize,usize,usize) {
	let (w,h) = (soluce.len_of(Axis(0)),soluce.len_of(Axis(1)));
	let mut xmin = usize::MAX;
	let mut ymin = usize::MAX;
	let mut xmax = 0;
	let mut ymax = 0;

	println!("{}",soluce);

	for y in 0..h {
		for x in 0..w {
			let cell = soluce.get((x,y)).unwrap();
			if cell.piece_id != NONE {
				if x < xmin {
					xmin = x;
				}
				if y < ymin {
					ymin = y;
				}
				if x > xmax {
					xmax = x;
				}
				if y > ymax {
					ymax = y;
				}
			} 
		}
	}

	println!("Sol : {} {} {} {}",xmin,ymin,xmax,ymax);
	(xmin,ymin,xmax - xmin + 1,ymax - ymin + 1)
}

fn draw_piece(image: &mut RgbaImage, piece: &Piece, rotate: usize,pos:(u32,u32),size:(u32,u32)) {
	//rotate image and
	let imgrot;
	let maskrot;
	match rotate {
		0 => {imgrot = piece.image.clone();maskrot = piece.mask.clone()},
		1 => {imgrot = imageops::rotate90(&piece.image); maskrot = imageops::rotate90(&piece.mask);},
		2 => {imgrot = imageops::rotate180(&piece.image); maskrot = imageops::rotate180(&piece.mask);},
		3 => {imgrot = imageops::rotate270(&piece.image); maskrot = imageops::rotate270(&piece.mask);},
		_ => panic!("Invalid value should be 0,1,2 or 3 !"),
	}

 	//recalculate rect
	let rect = extract_surrounding_rect(&maskrot);

	//compute middle pos
	let (xout0,yout0) = (pos.0 + (size.0 - (rect.2-rect.0)) / 2, pos.1 + (size.1 - (rect.3-rect.1)) / 2,);
	let (x0,y0,x1,y1) = rect;
	let (w,h) = (x1-x0,y1-y0);
	let keepcolor = Luma([common::MASK_PIECE_PIXEL]);

	//draw
	for y in 0..h {
		for x in 0..w {
			let xin = x + x0;
			let yin = y + y0;
			let m = maskrot.get_pixel(xin,yin);
			if *m == keepcolor {
				let color = imgrot.get_pixel(xin,yin);
				image.put_pixel(xout0+x,yout0+y,*color);
			}
		}
	}
}

pub fn draw_solution(soluce:&Soluce, pieces: &PieceVec,fname:&str) {
	//extract some infos
	let surround = build_surround(pieces);
	let (w_el,h_el) = max_size(&surround);
	let (x_sol,y_sol,w_sol,h_sol) = get_soluce_size(soluce);
	let (w,h) = (w_sol as u32 * w_el,h_sol as u32 * h_el);

	println!("Sol : {},{} * {},{}",w_el,h_el,w_sol,h_sol);

	//build final image
	let mut image = RgbaImage::new(w,h);
	let back = image::Rgba([255u8,255u8,255u8,0u8]);
	drawing::draw_filled_rect(&mut image,rect::Rect::at(0, 0).of_size(w, h),back);

	//draw all pieces in place
	for y in 0..h_sol {
		for x in 0..w_sol {
			let s = soluce.get((x+x_sol,y+y_sol)).unwrap();
			let pid = s.piece_id;
			if pid != NONE {
				let p = pieces[pid].lock().unwrap();
				let pos = (x as u32 * w_el,y as u32 * w_el);
				draw_piece(&mut image, &p, s.rotation,pos,(w_el,h_el));
			}
		}
	}

	//save into file
	image.save(fname).unwrap();
}