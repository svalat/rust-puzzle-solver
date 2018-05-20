/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// This module implement the rotation of the piece to put them in an optimized orientation for next algorithms.

//load external
extern crate image;
extern crate imageproc;

//from internal
use common;

//load std
use std::f32;
use std::cmp;

/// Calculate the coordinate of a line turned from a given angle and at offset distance of the
/// image center.
fn calc_line_coord(img:&image::GrayImage,angle:u32,offset:u32) -> ((f32,f32),(f32,f32)) {
	//middle of image
	let (w,h) = img.dimensions();
	let (xm,ym) = ((w/2) as f32,(h/2) as f32);
	let ray = xm.max(ym);

	//compute angle in radian
	let rad = (angle as f32).to_radians();

	//compute shift
	let of = offset as f32;
	let perp = rad + f32::consts::PI / 2.0;
	let (shift_x,shift_y) = (of * perp.cos(),of * perp.sin());

	//compute extrem points
	let opposite = rad + f32::consts::PI;
	let (x0,y0) = (xm+ray*(opposite).cos()+shift_x,ym+ray*(opposite).sin()+shift_y);
	let (x1,y1) = (xm+ray*(rad).cos()+shift_x,ym+ray*(rad).sin()+shift_y);

	//ret
	((x0,y0),(x1,y1))
}

/// Draw the line limit for debugging.
pub fn draw_limit_line(img:&mut image::GrayImage,angle:u32,offset:u32) {
	//coords
	let (start,end) = calc_line_coord(&img,angle,offset);

	//draw line
	imageproc::drawing::draw_line_segment_mut(img,start,end,image::Luma([common::MASK_SURROUND_RECT]));
}

#[allow(dead_code)]
fn calc_touched_points(img: &image::GrayImage,angle:u32,offset:u32) -> u32 {
	//coords
	let (start,end) = calc_line_coord(&img,angle,offset-1);
	let mut cnt = 0;

	//check if has something
	let color = image::Luma([common::MASK_PIECE_PIXEL]);
	let iter = imageproc::drawing::BresenhamLineIter::new(start,end);
	let (w,h) = img.dimensions();
	let (iw,ih) = (w as i32, h as i32);
	let mut last = -1;
	for (x,y) in iter {
		if x >= 0 && x < iw && y >= 0 && y < ih {
			if *img.get_pixel(x as u32,y as u32) == color {
				//cnt += 1;
				if last >= 40 {
					cnt += 1;
				}
				last = 0;
			} else {
				if last >= 0 {
					last += 1;
				}
			}
		}
	}

	cnt
}

/// Check if the given line cover some interestsing pixel so we can consider searching the next
/// one to build the rectangle surrounding the object to rotate.
fn check_limit_line(img:&image::GrayImage,angle:u32,offset:u32) -> bool {
	//coords
	let (start,end) = calc_line_coord(&img,angle,offset);

	//check if has something
	let mut has_pixel = false;
	let color = image::Luma([common::MASK_PIECE_PIXEL]);
	let iter = imageproc::drawing::BresenhamLineIter::new(start,end);
	let (w,h) = img.dimensions();
	let (iw,ih) = (w as i32, h as i32);
	for (x,y) in iter {
		if x >= 0 && x < iw && y >= 0 && y < ih {
			if *img.get_pixel(x as u32,y as u32) == color {
				has_pixel = true;
				break;
			}
		}
	}

	//ret
	has_pixel
}

/// For a given angle search the min distance for which the line do not cover any
/// interesting pixels.
fn find_limit_offset(img:&image::GrayImage,angle:u32) -> u32 {
	//get max offset
	let (w,h) = img.dimensions();
	let max = cmp::max(w,h)/2;

	//loop
	let mut ret = 0;
	for offset in 1..max {
		if !check_limit_line(&img,angle,offset) {
			ret = offset;
			break;
		}
	}

	//ret
	ret
}

/// Draw the best rectangle for debugging using the given angle.
pub fn draw_best_rectangle(img:&mut image::GrayImage,angle:u32) {
	//axis 1
	let offset1 = find_limit_offset(&img,angle);
	let offset2 = find_limit_offset(&img,angle+180);

	//axis2
	let offset3 = find_limit_offset(&img,angle+90);
	let offset4 = find_limit_offset(&img,angle+90+180);

	//draw
	draw_limit_line(img,angle,offset1);
	draw_limit_line(img,angle+180,offset2);
	draw_limit_line(img,angle+90,offset3);
	draw_limit_line(img,angle+90+180,offset4);
}

/// Find the best rectangle to rotate the piece by considering searching the
/// larger ratio between larger side and smaller side of rectangle.
/// It return the bests angle.
pub fn find_best_rectangle(img:&image::GrayImage) -> u32 {
	let mut angle:u32 = 0;
	let mut angle_max = 0;
	let mut ratio_max = 0f32;
	while angle <= 90 {
		//axis 1
		let offset1 = find_limit_offset(&img,angle);
		let offset2 = find_limit_offset(&img,angle+180);

		//axis2
		let offset3 = find_limit_offset(&img,angle+90);
		let offset4 = find_limit_offset(&img,angle+90+180);

		//touched pixels
		//let touched1 = calc_touched_points(&img,angle,offset1);
		//let touched2 = calc_touched_points(&img,angle+180,offset2);
		//let touched3 = calc_touched_points(&img,angle+90,offset3);
		//let touched4 = calc_touched_points(&img,angle+90+180,offset4);

		//compute rect
		let axe1 = offset1 + offset2;
		let axe2 = offset3 + offset4;

		//built canonical rect
		let axe_min = cmp::min(axe1,axe2);
		let axe_max = cmp::max(axe1,axe2);

		//ratio
		let ratio = axe_max as f32/axe_min as f32;
		//let ratio = (touched1 + touched2 + touched3 + touched4) as f32;
		//println!("--------- RATIO {}",ratio);

		//extract max
		if ratio > ratio_max {
			ratio_max = ratio;
			angle_max = angle;
		}

		//inc for next step
		angle = angle + common::ROTATE_ANGLE_RESOLUTION;
	}

	//println!("--------- RATIO {}",ratio_max);

	//ret
	angle_max
}

/// Finally appply the rotation to get the image in right position
pub fn do_rotate_gray(img:& image::GrayImage,angle:u32) -> image::GrayImage {
	//calc center
	let (w,h) = img.dimensions();
	let center = ((w / 2) as f32,(h / 2) as f32);

	//angle in radian
	let angle: f32 = (-(angle as f32)).to_radians();

	//rotate
	//for mask we don't want interpolation to keep exact color for matchin
	imageproc::affine::rotate(img,center,angle,imageproc::affine::Interpolation::Nearest)
}

/// Finally appply the rotation to get the image in right position
pub fn do_rotate_rgba(img:& image::RgbaImage,angle:u32) -> image::RgbaImage {
	//calc center
	let (w,h) = img.dimensions();
	let center = ((w / 2) as f32,(h / 2) as f32);

	//angle in radian
	let angle: f32 = (-(angle as f32)).to_radians();

	//rotate
	imageproc::affine::rotate(img,center,angle,imageproc::affine::Interpolation::Bilinear)
}