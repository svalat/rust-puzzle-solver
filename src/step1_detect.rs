/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// This module is used to implement the search of the pieces in the picture.

//load external
extern crate image;

///Search first pixel which is not color of background. Starting from top left
///So fount the most top and most left pixel it found (top first).
///
/// **img** The image into which to search
/// **back** The color of the background to ignore the related pixels
pub fn find_first_non_bg_pixel(img:&image::RgbaImage,back:&image::Rgba<u8>) -> (u32,u32)
{
	let blue = image::Rgba([0 as u8,0 as u8,255 as u8,255 as u8]);
	let green = image::Rgba([0 as u8,255 as u8,0 as u8,255 as u8]);
	let mut ret = (0,0);
	for (x,y,color) in img.enumerate_pixels() {
		if color != back && *color != blue && *color != green {
			ret = (x,y);
			println!("PIXEL : {:?} {:?}",(x,y),color);
			break;
		}
	}
	ret
}

/// Search the limit of the object on the give line. It loop from the starting point the one side
/// (depening on the sign of offset). When it don't see pixels anymore it stop and return
/// the position.
///
/// **img**: image on which to perform the search.
/// **back**: background color of pixels to ignore.
/// **first**: starting point, from this we search on left and right based on offset
/// **offset**: -1 to search on left, 1 to search on right.
fn find_limit(img:&image::RgbaImage,back:&image::Rgba<u8>,first:(u32,u32),offset:i32) -> u32
{
	//check
	debug_assert!(offset == -1 || offset ==1);
	debug_assert!(first.0 < img.dimensions().0);
	debug_assert!(first.1 < img.dimensions().1);

	//colors to ignore
	let blue = image::Rgba([0 as u8,0 as u8,255 as u8,255 as u8]);
	let green = image::Rgba([0 as u8,255 as u8,0 as u8,255 as u8]);
	
	//vars
	let mut done = false;
	let (xuint,y) = first;
	let mut x:i32 = xuint as i32;
	let mut ret:u32 = x as u32;
	let (xmax,_) = img.dimensions();

	//loop unitl end
	while !done {
		let color = img.get_pixel(x as u32,y);

		//if has interesting pixel
		if color != back && *color != blue && *color != green {
			ret = x as u32;
			x = x + offset;
		} else {
			done = true;
		}

		//out of picture
		if x < 0 || x >= xmax as i32 {
			done = true;
		}
	}

	//return
	ret
}

/// Check if a given line contain pixel to know if we reach the bottom of the object to extract
/// xmin and xmax should be the left and right limit of the object.
///
/// **img**: image in which to check the pixels
/// **back**: background color to ignore
/// **y**: coordinate of line on Y axis (top/bottom)
/// **xmin**: Start to search at this point on X axis.
/// **xmax**: Finish to search at this point on X axis.
fn check_has_pixel_on_line(img:&image::RgbaImage,back:&image::Rgba<u8>,y:u32,xmin:u32,xmax:u32) -> bool {
	//checks
	debug_assert!(y < img.dimensions().1);
	debug_assert!(xmin < img.dimensions().0);
	debug_assert!(xmax < img.dimensions().0);

	//vars
	let blue = image::Rgba([0 as u8,0 as u8,255 as u8,255 as u8]);
	let green = image::Rgba([0 as u8,255 as u8,0 as u8,255 as u8]);
	let mut ret = false;
	let xmax = xmax + 1;

	//loop
	for x in xmin..xmax {
		let color = img.get_pixel(x,y);
		if color != back && *color != blue && *color != green {
			ret = true;
			break;
		}
	}

	ret
}

/// Start from the first interesting pixel and compute the rectangle surrounding the object.
/// It proceed by searching limits on all lines going from top to bottom until it doesn't
/// anymore find interesting pixels one the line.
///
/// **img**: image in which to search
/// **back**: backgroudn color to ignore
/// **first**: Starting point from where to build surounding rect of shape.
pub fn find_square_non_bg(img:&image::RgbaImage,back:&image::Rgba<u8>,first:(u32,u32)) -> (u32,u32,u32,u32)
{
	let (_,ymin) = first;
	let (x,mut y) = first;
	let (_,ymax) = img.dimensions();
	let mut done = false;
	let (mut xmin,mut xmax) = (x,x);

	while !done {
		//seach on left to extend
		xmin = find_limit(&img,&back,(xmin,y),-1);

		//search on right to extend
		xmax = find_limit(&img,&back,(xmax,y),1);

		//check if has somehting on the line between xmin and xmax
		if !check_has_pixel_on_line(&img,&back,y,xmin,xmax) {
			done = true;
		}

		//inc and we stop if we reach end
		y = y + 1;
		if y == ymax {
			done = true;
		}
	}

	(xmin,ymin,xmax-xmin,y-ymin)
}

/// Paint the square for debugging and to ignore the object for next sarting point search.
/// It paint the background in green and the object in blue.
pub fn paint_square(img:&mut image::RgbaImage,back:&image::Rgba<u8>,square:(u32,u32,u32,u32)) {
	let (xmin,ymin,sx,sy) = square;
	let xmax = xmin+sx+1;
	let ymax = ymin+sy;

	for y in ymin..ymax {
		for x in xmin..xmax {
			let mut pix = img.get_pixel_mut(x,y);
			if pix == back {
				pix.data = [0,255,0,255];
			} else {
				pix.data = [0,0,255,255];
			}
		}
	}
}

///Implement unit tests of the module
#[cfg(test)]
mod test {
	use image;
	use Path;
	use step1_detect;

	#[test]
	fn find_first_non_bg_pixel() {
		let img = image::open(&Path::new("tests/trivial-1.png")).unwrap();
		let img = img.as_rgba8().unwrap();
		let res = step1_detect::find_first_non_bg_pixel(&img,img.get_pixel(0,0));
		assert_eq!(res,(85,42));
	}

	#[test]
	fn find_limit()
	{
		let img = image::open(&Path::new("tests/trivial-1.png")).unwrap();
		let img = img.as_rgba8().unwrap();
		let left = step1_detect::find_limit(&img,img.get_pixel(0,0),(95,42),-1);
		let right = step1_detect::find_limit(&img,img.get_pixel(0,0),(95,42),1);
		assert_eq!(left,85);
		assert_eq!(right,227);
	}

	#[test]
	fn check_has_pixel_on_line() {
		let img = image::open(&Path::new("tests/trivial-1.png")).unwrap();
		let img = img.as_rgba8().unwrap();
		let ret1 = step1_detect::check_has_pixel_on_line(&img,img.get_pixel(0,0),0,85,227);
		let ret2 = step1_detect::check_has_pixel_on_line(&img,img.get_pixel(95,42),0,85,227);
		assert_eq!(ret1,false);
		assert_eq!(ret2,true);
	}

	#[test]
	fn find_square_non_bg() {
		let img = image::open(&Path::new("tests/trivial-1.png")).unwrap();
		let img = img.as_rgba8().unwrap();
		let ret = step1_detect::find_square_non_bg(&img,img.get_pixel(0,0),(85,43));
		assert_eq!(ret,(85,43,142,174));
	}
}