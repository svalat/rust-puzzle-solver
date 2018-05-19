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
use std::f32;
use std::u32;
use std::cmp;

fn find_first_bump_point(img: &image::GrayImage,start:(u32,u32),step:(u32,u32),steps:u32) -> (u32,u32) {
	let color = image::Luma([common::MASK_HIDDEN_BUMP]);
	let mut found: Vec<u32> = vec!();
	let mut coord = (0,0);
	
	for i in 0..steps {
		let x = start.0 + i * step.0;
		let y = start.1 + i * step.1;

		if *img.get_pixel(x,y) == color {
			found.push(i);
		}
	}

	//take middle point
	if !found.is_empty() {
		let index = found.len() / 2;
		let i = found[index];
		let x = start.0 + i * step.0;
		let y = start.1 + i * step.1;
		coord = (x,y);
	}

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
		PieceSideType::Bump =>	points.top_shape = find_first_bump_point(img,(xmin,ymin),(1,0),w),
		PieceSideType::Hole => {},
		PieceSideType::Unknown => panic!("Should not get Unknwon here !"),
	}

	//left
	match side_infos.left {
		PieceSideType::Bump =>	points.left_shape = find_first_bump_point(img,(xmin,ymin),(0,1),h),
		PieceSideType::Hole => {},
		PieceSideType::Unknown => panic!("Should not get Unknwon here !"),
	}

	//right
	match side_infos.right {
		PieceSideType::Bump =>	points.right_shape = find_first_bump_point(img,(xmax,ymin),(0,1),h),
		PieceSideType::Hole => {},
		PieceSideType::Unknown => panic!("Should not get Unknwon here !"),
	}

	//bottom
	match side_infos.bottom {
		PieceSideType::Bump =>	points.bottom_shape = find_first_bump_point(img,(xmin,ymax),(1,0),w),
		PieceSideType::Hole => {},
		PieceSideType::Unknown => panic!("Should not get Unknwon here !"),
	}

	//print
	println!("Points : {:?}",points);	
}