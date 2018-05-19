/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// Search the bump/hole extrem points. This will be usefull to pre-match the sides
/// before doing pixel matching with the two pictures.

//load external
extern crate image;

//internal
use common;
use step5_corners;
use piece::{PiecePoints,PieceSideInfos,PieceSideType};

//std
use std::u32;

fn make_hole_scan(img: &image::GrayImage,start:(u32,u32),search:(i32,i32),search_steps:u32) -> ((u32,u32),u32) {
	//vars
	let color = image::Luma([common::MASK_BACKGROUND]);
	let (x,y) = (start.0 as i32,start.1 as i32);
	let (sx,sy) = search;
	let mut coord = start;
	let mut smax = 0;

	//loop
	for i in 0..search_steps {
		let x = (x + i as i32 * sx) as u32;
		let y = (y + i as i32 * sy) as u32;
		if *img.get_pixel(x,y) != color {
			coord = (x,y);
			smax = i;
			break;
		}
	}

	//ret
	(coord,smax)
}

fn find_hole_point(img: &image::GrayImage,start:(u32,u32),step:(u32,u32),search:(i32,i32),steps:u32,search_steps:u32) -> (u32,u32) {
	//vars
	let mut found: Vec<(u32,u32)> = vec!();
	let mut coord = (0,0);
	let mut max_dist = 0;
	let (x,y) = start;
	let (sx,sy) = step;

	//loop
	for i in 0..steps {
		let start = (x + i * sx,y + i * sy);
		
		let (coord,dist) = make_hole_scan(img,start,search,search_steps);

		if dist > max_dist {
			max_dist = dist;
			found.clear();
			found.push(coord);
		} else if dist == max_dist {
			found.push(coord);
		}
	}

	//take middle point
	if !found.is_empty() {
		let index = found.len() / 2;
		coord = found[index];
	}

	//ret
	coord
}

fn find_bump_point(img: &image::GrayImage,start:(u32,u32),step:(u32,u32),steps:u32) -> (u32,u32) {
	let color = image::Luma([common::MASK_HIDDEN_BUMP]);
	let mut found: Vec<(u32,u32)> = vec!();
	let mut coord = (0,0);
	
	for i in 0..steps {
		let x = start.0 + i * step.0;
		let y = start.1 + i * step.1;

		if *img.get_pixel(x,y) == color {
			found.push((x,y));
		}
	}

	//take middle point
	if !found.is_empty() {
		let index = found.len() / 2;
		coord = found[index];
	}

	//ret
	coord
}

pub fn draw_corners(img: &mut image::GrayImage,points: &PiecePoints) {
	//point
	step5_corners::draw_point(img,points.top_shape);
	step5_corners::draw_point(img,points.bottom_shape);
	step5_corners::draw_point(img,points.left_shape);
	step5_corners::draw_point(img,points.right_shape);
}

pub fn extract_piece_points(img: &image::GrayImage,points: &mut PiecePoints,side_infos: &PieceSideInfos) {
	//extract surrounding rect
	let (xmin,ymin,xmax,ymax) = step5_corners::extract_surrounding_rect(img);
	let (w,h) = ((xmax - xmin),(ymax-ymin));

	//top
	match side_infos.top {
		PieceSideType::Bump =>	points.top_shape = find_bump_point(img,(xmin,ymin),(1,0),w),
		PieceSideType::Hole => points.top_shape = find_hole_point(img,
			(points.top_left_corner.0,ymin),
			(1,0),(0,1),
			points.top_right_corner.0 - points.top_left_corner.0,
			h
		),
		PieceSideType::Unknown => panic!("Should not get Unknwon here !"),
	}

	//left
	match side_infos.left {
		PieceSideType::Bump =>	points.left_shape = find_bump_point(img,(xmin,ymin),(0,1),h),
		PieceSideType::Hole => points.left_shape = find_hole_point(img,
			(xmin,points.top_left_corner.1),
			(0,1),(1,0),
			points.bottom_left_corner.1 - points.top_left_corner.1,
			w
		),
		PieceSideType::Unknown => panic!("Should not get Unknwon here !"),
	}

	//right
	match side_infos.right {
		PieceSideType::Bump =>	points.right_shape = find_bump_point(img,(xmax,ymin),(0,1),h),
		PieceSideType::Hole => points.right_shape = find_hole_point(img,
			(xmax,points.top_right_corner.1),
			(0,1),(-1,0),
			points.bottom_right_corner.1 - points.top_right_corner.1,
			w
		),
		PieceSideType::Unknown => panic!("Should not get Unknwon here !"),
	}

	//bottom
	match side_infos.bottom {
		PieceSideType::Bump =>	points.bottom_shape = find_bump_point(img,(xmin,ymax),(1,0),w),
		PieceSideType::Hole => points.bottom_shape = find_hole_point(img,
			(points.bottom_left_corner.0,ymax),
			(1,0),(0,-1),
			points.bottom_right_corner.0 - points.bottom_left_corner.0,
			h
		),
		PieceSideType::Unknown => panic!("Should not get Unknwon here !"),
	}

	//print
	println!("Points : {:?}",points);	
}