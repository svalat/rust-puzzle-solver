/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

//load external
extern crate image;

//from image
use image::GrayImage;
use image::RgbaImage;

//consts
const EXTRACT_MARGINS: u32 = 30;

pub struct Piece {
	pub id: u32,
	pub position:(u32,u32,u32,u32),
	pub image: RgbaImage,
	pub mask: GrayImage,
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
