/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

//load external
extern crate image;
extern crate imageproc;

//load std
use std::f32;
use std::cmp;

//consts
const ANGLE_RESOLUTION: u32 = 1;

/// Calculate the coordinate of a line turned from a given angle and at offset distance of the
/// image center.
fn calc_line_coord(img:&image::GrayImage,angle:u32,offset:u32) -> ((f32,f32),(f32,f32)) {
	//middle of image
	let (w,h) = img.dimensions();
	let (xm,ym) = ((w/2) as f32,(h/2) as f32);

	//compute angle in radian
	let rad = (angle as f32).to_radians();

	//compute shift
	let of = offset as f32;
	let perp = rad + f32::consts::PI / 2.0;
	let (shift_x,shift_y) = (of * perp.cos(),of * perp.sin());

	//compute extrem points
	let opposite = rad + f32::consts::PI;
	let (x0,y0) = (xm+xm*(opposite).cos()+shift_x,ym+ym*(opposite).sin()+shift_y);
	let (x1,y1) = (xm+xm*(rad).cos()+shift_x,ym+ym*(rad).sin()+shift_y);

	//ret
	((x0,y0),(x1,y1))
}

/// Draw the line limit for debugging.
pub fn draw_limit_line(img:&mut image::GrayImage,angle:u32,offset:u32) {
	//coords
	let (start,end) = calc_line_coord(&img,angle,offset);

	//draw line
	imageproc::drawing::draw_line_segment_mut(img,start,end,image::Luma([255 as u8]));
}

/// Check if the given line cover some interestsing pixel so we can consider searching the next
/// one to build the rectangle surrounding the object to rotate.
fn check_limit_line(img:&image::GrayImage,angle:u32,offset:u32) -> bool {
	//coords
	let (start,end) = calc_line_coord(&img,angle,offset);

	//check if has something
	let mut has_pixel = false;
	let color = image::Luma([128 as u8]);
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

		//compute rect
		let axe1 = offset1 + offset2;
		let axe2 = offset3 + offset4;

		//built canonical rect
		let axe_min = cmp::min(axe1,axe2);
		let axe_max = cmp::max(axe1,axe2);

		//ratio
		let ratio = axe_max as f32/axe_min as f32;

		//extract max
		if ratio > ratio_max {
			ratio_max = ratio;
			angle_max = angle;
		}

		//inc for next step
		angle = angle + ANGLE_RESOLUTION;
	}

	//ret
	angle_max
}
