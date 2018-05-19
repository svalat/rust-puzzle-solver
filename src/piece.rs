/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

//load external
extern crate image;

//from internal
use common;

//from image
use image::GrayImage;
use image::RgbaImage;

//consts
const EXTRACT_MARGINS: u32 = 30;

/// Structure to remember which side is bump and which is hole (true for bump)
#[derive(Debug)]
pub struct PieceSideInfos {
	pub left: bool,
	pub right: bool,
	pub top: bool,
	pub bottom: bool,
}

/// Define a piece of the puzzle, this consist in an ID, a position in the global picture (rectangle)
/// and the extracted image of the peice in color with margins to later rotate it. It also contain
/// a mask of the piece in gray color to faster (instead of using RGB) scan the piece for all later 
/// operation up the the matching.
pub struct Piece {
	pub id: u32,
	pub position:(u32,u32,u32,u32),
	pub image: RgbaImage,
	pub mask: GrayImage,
	pub angle: u32,
	pub side_infos: PieceSideInfos,
}

impl PieceSideInfos {
	/// Constructor to init side infos
	pub fn new() -> Self {
		Self {
			left: false,
			right: false,
			top: false,
			bottom: false,
		}
	}
}

impl Piece {
	/// Constructor of a puzzle piece. It take the global image, the backround color and a square
	/// from which to extract the image and the mask.
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
			angle: 0,
			side_infos: PieceSideInfos::new(),
		};

		//init images
		for pixel in cur.image.pixels_mut() {
			*pixel = *back;
		}
		let b = image::Luma([common::MASK_BACKGROUND]);
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
		let col = image::Luma([common::MASK_PIECE_PIXEL]);
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

	/// Save the image and mask into files for debugging.
	pub fn save(self: &Piece, step: u32, name: &str) {
		//build base name
		let base = format!("step-{}-{}-{}",step,name,self.id);

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
