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
use std::env;
use std::fs::File;
use std::path::Path;
use std::f32;
use std::cmp;

use image::GenericImage;
use image::GrayImage;
use image::RgbaImage;

//use imageproc::drawing;

const EXTRACT_MARGINS: u32 = 30;
const ANGLE_RESOLUTION: u32 = 1;

pub struct Piece {
	id: u32,
	position:(u32,u32,u32,u32),
	image: RgbaImage,
	mask: GrayImage,
}

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

fn draw_limit_line(img:&mut image::GrayImage,angle:u32,offset:u32) {
	//coords
	let (start,end) = calc_line_coord(&img,angle,offset);

	//draw line
	imageproc::drawing::draw_line_segment_mut(img,start,end,image::Luma([255 as u8]));
}

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

fn draw_best_rectangle(img:&mut image::GrayImage,angle:u32) {
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

fn find_best_rectangle(img:&image::GrayImage) -> u32 {
	let (w,h) = img.dimensions();
	let max = cmp::max(w,h) / 2;

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

impl Piece {
	pub fn new(img:&image::RgbaImage,back:&image::Rgba<u8>,square:(u32,u32,u32,u32),id:u32) -> Self {
		//extract
		let (x0,y0,w,h) = square;

		//take margins to rotate inside
		let (ww,hh) = (w+2*EXTRACT_MARGINS*w/100,h+2*EXTRACT_MARGINS*h/100);
		let (x1,y1) = (EXTRACT_MARGINS*w/100,EXTRACT_MARGINS*h/100);

		//create
		let mut cur = Piece {
			id: id,
			position: square,
			image: RgbaImage::new(ww,hh),
			mask: GrayImage::new(ww,hh),
		};

		//init images
		for pixel in cur.image.pixels_mut() {
			*pixel = *back;
		}
		let b = image::Luma([0 as u8]);
		for pixel in cur.mask.pixels_mut() {
			*pixel = b;
		}

		//copy image & mask
		for y in 0..h {
			for x in 0..w {
				cur.image.put_pixel(x1+x,y1+y,*img.get_pixel(x0+x,y0+y));
			}
		}

		//build mask
		let col = image::Luma([128u8]);
		for y in 0..h {
			for x in 0..w {
				if img.get_pixel(x0+x,y0+y) != back {
					cur.mask.put_pixel(x1+x,y1+y,col);
				}
			}
		}

		//ret
		cur
	}

	pub fn save(self: &Piece) {
		//build base name
		let base = format!("step-2-extract-{:?}",self.id);

		//export image
		{
			let fname = base.to_string() + "-img.png";
			self.image.save(fname).unwrap();
		}

		//export mask
		{
			let fname = base.to_string() + "-mask.png";
			self.mask.save(fname).unwrap();
		}
	}
}

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

	//check if RGB which is not supported by our code
	{
		let r = img.as_mut_rgb8();
		match r {
			Some(_) => println!("Not ok, image if RGB, not RGBA"),
			None => {}
		}
	}

	//list
	let mut all: Vec<Piece> = Vec::new();

	//find first black pixel
	{
		let rgba8 = img.as_mut_rgba8();
		match rgba8 {
			Some(rgba) => {
				let mut first = (1,1);
				let mut id: u32 = 0;
				while first != (0,0) {
					first = find_first_non_bg_pixel(&rgba,&background);
					println!("First black pixel : {:?}",first);

					//fin square
					let square = find_square_non_bg(&rgba,&background,first);
					println!("Square : {:?}",square);

					//extract into list
					let (_,_,w,h) = square;
					if w*h > 100 {
						all.push(Piece::new(rgba,&background,square,id));
						id = id + 1;
					} else {
						println!("IGNORE, too small !");
					}
					
					//draw for save
					paint_square(rgba,&background,square);
				}
			},
			None => println!("Invalid format ! Expect RGBA8 !")
		}
	}

	//down line
	for p in all.iter_mut() {
		
		println!("------ {:?} ------",p.id);
		let angle = find_best_rectangle(&p.mask);
		println!("+++> {:?}",angle);
		draw_best_rectangle(&mut p.mask,angle);

		/*let angle = 0;
		let offset1 = find_limit_offset(&p.mask,angle);
		let offset2 = find_limit_offset(&p.mask,angle+180);
		draw_limit_line(&mut p.mask,angle,offset1);
		draw_limit_line(&mut p.mask,angle+180,offset2);*/
	}

	//save list
	for p in all.iter() {
		p.save();
	}

	//create output image
	let ref mut fout = File::create("step-1-detect.png").unwrap();

	//write to file
	img.write_to(fout, image::PNG).unwrap();
}
