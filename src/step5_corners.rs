/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// Search the corners. This will be usefull to pre-match the sides
/// before doing pixel matching with the two pictures.

//load external
extern crate image;

//internal
use common;
use piece::PiecePoints;

//std
use std::f32;
use std::u32;
use std::cmp;

//search on the line to find point closer to ref
fn search_closer_point_on_line(img: &image::GrayImage,ref_point: (u32,u32),start: (u32,u32),step:(i32,i32),steps: u32) -> ((u32,u32),f32) {
	//check
	assert!(step.0 == 0 || step.1 == 0);

	//infos
	let (w,h) = img.dimensions();
	let mut min_dist: f32 = f32::MAX;
	let mut coord: (u32,u32) = (u32::MAX,u32::MAX);
	let (x0,y0) = (start.0 as i32,start.1 as i32);
	let (rx0,ry0) = (ref_point.0 as i32,ref_point.1 as i32);
	let color = image::Luma([common::MASK_PIECE_PIXEL]);

	//loop
	for i in 0..steps {
		//calc pixel pos
		let x: i32 = x0 + step.0 * i as i32; 
		let y: i32 = y0 + step.1 * i as i32;

		//check if in image
		if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
			//convert
			let ux: u32 = x as u32;
			let uy: u32 = y as u32;

			//check pixel
			if *img.get_pixel(ux,uy) == color {
				//calc distance
				let dx = x - rx0;
				let dy = y - ry0;
				let dist = ((dx*dx+dy*dy) as f32).sqrt();

				//check if keep
				if dist < min_dist {
					min_dist = dist;
					coord = (ux,uy);
				}
			}
		}
	}

	(coord,min_dist)
}

//start from the given point and scan squares from this and stop when it found the pixel
//with shorter distance to the starting point
fn search_corner(img: &image::GrayImage,start: (u32,u32),step: (i32,i32),steps: u32) -> (u32,u32) {
	//vars
	let mut min_dist: f32 = f32::MAX;
	let mut coord: (u32,u32) = (u32::MAX,u32::MAX);
	let (x0,y0) = start;
	let (sx,sy) = step;

	//do all steps
	for i in 0..steps {
		//coord
		let x = (x0 as i32 + i as i32 * sx) as u32;
		let y = (y0 as i32 + i as i32 * sy) as u32;

		//loop on horizontal line
		let (c,d) = search_closer_point_on_line(img,(x0,y0),(x0,y),(sx,0),i);
		if d < min_dist {
			coord = c;
			min_dist = d;
		}

		//loop on vertical line
		let (c,d) = search_closer_point_on_line(img,(x0,y0),(x,y0),(0,sy),i);
		if d < min_dist {
			coord = c;
			min_dist = d;
		}
	}

	coord
}

pub fn extract_surrounding_rect(img: &image::GrayImage) -> (u32,u32,u32,u32) {
	//vars
	let mut xmin = u32::MAX;
	let mut xmax = 0;
	let mut ymin = u32::MAX;
	let mut ymax = 0;
	let color1 = image::Luma([common::MASK_PIECE_PIXEL]);
	let color2 = image::Luma([common::MASK_HIDDEN_BUMP]);
	let (w,h) = img.dimensions();

	//loop on all pixels
	for x in 0..w {
		for y in 0..h {
			let p = img.get_pixel(x,y);
			if *p == color1 || *p == color2 {
				if x < xmin {
					xmin = x;
				}
				if x > xmax {
					xmax = x;
				}
				if y < ymin {
					ymin = y;
				}
				if y > ymax {
					ymax = y;
				}
			}
		}
	}

	//ret
	(xmin,ymin,xmax,ymax)
}

pub fn draw_point(img: &mut image::GrayImage,coord: (u32,u32)) {
	//vars
	let pinterest = image::Luma([common::MASK_POINTS_INTREST]);
	let pback = image::Luma([common::MASK_POINTS_BACK]);
	let color = image::Luma([common::MASK_BACKGROUND]);

	//draw
	if coord.0 != u32::MAX && coord.1 != u32::MAX {
		let nb:i32 = 5;
		for x in -nb..nb {
			for y in -nb..nb {
				let x = (coord.0 as i32+x) as u32;
				let y = (coord.1 as i32+y) as u32;
				if *img.get_pixel(x,y) == color {
					img.put_pixel(x,y,pback);
				} else {
					img.put_pixel(x,y,pinterest);
				}
			}
		}
	}
}

pub fn draw_corners(img: &mut image::GrayImage,points: &PiecePoints) {
	//point
	draw_point(img,points.top_left_corner);
	draw_point(img,points.top_right_corner);
	draw_point(img,points.bottom_left_corner);
	draw_point(img,points.bottom_right_corner);
}

pub fn extract_piece_points(img: &image::GrayImage) -> PiecePoints {
	//vars
	let mut points = PiecePoints::new();

	//extract surrounding rect
	let (xmin,ymin,xmax,ymax) = extract_surrounding_rect(img);
	let steps = cmp::min(xmax-xmin,ymax-ymin) / 2;

	println!("Rect {:?}",(xmin,ymin,xmax,ymax));

	//search them
	points.top_left_corner = search_corner(img,(xmin,ymin),(1,1),steps);
	points.top_right_corner = search_corner(img,(xmax,ymin),(-1,1),steps);
	points.bottom_left_corner = search_corner(img,(xmin,ymax),(1,-1),steps);
	points.bottom_right_corner = search_corner(img,(xmax,ymax),(-1,-1),steps);

	//print
	println!("Points : {:?}",points);	

	//return
	points
}