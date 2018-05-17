/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

//load external
extern crate image;

//load std
use std::env;
use std::fs::File;
use std::path::Path;

use image::GenericImage;

///Search first pixel which is not color of background
fn find_first_non_bg_pixel(img:&image::RgbaImage,back:&image::Rgba<u8>) -> (u32,u32)
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

fn find_limit(img:&image::RgbaImage,back:&image::Rgba<u8>,first:(u32,u32),offset:i32) -> u32
{
	let blue = image::Rgba([0 as u8,0 as u8,255 as u8,255 as u8]);
	let green = image::Rgba([0 as u8,255 as u8,0 as u8,255 as u8]);
	let mut done = false;
	let (xuint,y) = first;
	let mut x:i32 = xuint as i32;
	let mut ret:u32 = x as u32;
	let (xmax,_) = img.dimensions();

	while !done {
		let color = img.get_pixel(x as u32,y);
		if color != back && *color != blue && *color != green {
			ret = x as u32;
			x = x + offset;
		} else {
			done = true;
		}
		if x < 0 || x >= xmax as i32 {
			done = true;
		}
	}

	ret
}

fn check_has_pixel_on_line(img:&image::RgbaImage,back:&image::Rgba<u8>,y:u32,xmin:u32,xmax:u32) -> bool {
	let blue = image::Rgba([0 as u8,0 as u8,255 as u8,255 as u8]);
	let green = image::Rgba([0 as u8,255 as u8,0 as u8,255 as u8]);
	let mut ret = false;

	for x in xmin..xmax {
		let color = img.get_pixel(x,y);
		if color != back && *color != blue && *color != green {
			ret = true;
			break;
		}
	}

	ret
}

///search orig and size
fn find_square_non_bg(img:&image::RgbaImage,back:&image::Rgba<u8>,first:(u32,u32)) -> (u32,u32,u32,u32)
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

fn paint_square(img:&mut image::RgbaImage,back:&image::Rgba<u8>,square:(u32,u32,u32,u32)) {
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

fn main() {
	//info
	println!("rust-puzzle-solver-0.1.0");

	//load path
	let file = if env::args().count() == 2 {
        env::args().nth(1).unwrap()
    } else {
        panic!("Please enter a file")
    };

	//load image
	println!("Load image {:?}",file);
	let mut img = image::open(&Path::new(&file)).unwrap();

	//print size
	println!("dimensions {:?}", img.dimensions());

	//print colors
	println!("{:?}", img.color());

	//pixel 0,0
	let background = img.get_pixel(0,0);
	println!("Pixel 0,0 : {:?}",background);

	//find first black pixel
	{
		{
		let r = img.as_mut_rgb8();
		match r {
			Some(t) => println!("ok"),
			None => println!("nok")
		}
		}
		let rgba8 = img.as_mut_rgba8();
		match rgba8 {
			Some(rgba) => {
				let mut first = (1,1);
				while first != (0,0) {
					first = find_first_non_bg_pixel(&rgba,&background);
					println!("First black pixel : {:?}",first);

					//fin square
					let square = find_square_non_bg(&rgba,&background,first);
					println!("Square : {:?}",square);
					
					//draw for save
					paint_square(rgba,&background,square);
					
					//say if keep
					let (_,_,w,h) = square;
					if w*h < 100 {
						println!("IGNORE, too small !");
					}
				}
			},
			None => println!("Invalid format ! Expect RGBA8 !")
		}
	}

	//create output image
	let ref mut fout = File::create("output.png").unwrap();

	//write to file
	img.write_to(fout, image::PNG).unwrap();
}
