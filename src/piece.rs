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

//std
use std::u32;

//from image
use image::GrayImage;
use image::RgbaImage;
use std::mem;
use std::sync::{Mutex,Arc};

//consts
const EXTRACT_MARGINS: u32 = 30;
pub const TOP:usize = 0;
pub const RIGHT:usize = 1;
pub const BOTTOM:usize = 2;
pub const LEFT:usize = 3;

//side type
#[derive(Debug,Copy, Clone, PartialEq)]
pub enum PieceSideType {
	Hole,
	Bump,
	Unknown,
}

/// Structure to remember which side is bump and which is hole (true for bump)
#[derive(Debug)]
pub struct PieceSideInfos {
	pub left: PieceSideType,
	pub right: PieceSideType,
	pub top: PieceSideType,
	pub bottom: PieceSideType,
}

/// Coordinate for intersting points
#[derive(Debug)]
pub struct PiecePoints {
	pub top_left_corner: (u32,u32),
	pub top_right_corner: (u32,u32),
	pub bottom_left_corner: (u32,u32),
	pub bottom_right_corner: (u32,u32),
	pub top_shape: (u32,u32),
	pub bottom_shape: (u32,u32),
	pub right_shape: (u32,u32),
	pub left_shape: (u32,u32),
}

/// Store a matching solution
#[derive(Clone,Debug)]
pub struct PieceMatch {
	pub piece: usize,
	pub side: usize,
	pub angle: f32,
	pub distance: f32,
}

/// list of matching
pub type PieceMatchVec = Vec<PieceMatch>;

/// Prepared face parameters (similate to PiecePoints but pre-rotated and using f32 to ease usage possibly to compute rotations)
#[derive(Debug)]
pub struct PieceFace {
	pub top: (f32,f32),
	pub middle: (f32,f32),
	pub bottom: (f32,f32),
	pub mode: PieceSideType,
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
	pub points: PiecePoints,
	pub quality: u32,
	pub faces: [PieceFace;4],
	pub matches: [PieceMatchVec;4],
}

pub type PieceVec = Vec<Arc<Mutex<Piece>>>;

impl PiecePoints {
	/// Constructor to init coords
	pub fn new() -> Self {
		Self {
			top_left_corner: (u32::MAX,u32::MAX),
			top_right_corner: (u32::MAX,u32::MAX),
			bottom_left_corner: (u32::MAX,u32::MAX),
			bottom_right_corner: (u32::MAX,u32::MAX),
			top_shape: (u32::MAX,u32::MAX),
			bottom_shape: (u32::MAX,u32::MAX),
			right_shape: (u32::MAX,u32::MAX),
			left_shape: (u32::MAX,u32::MAX),
		}
	}
}

impl PieceFace {
	pub fn new() -> Self {
		Self {
			top: (0.0,0.0),
			middle: (0.0,0.0),
			bottom: (0.0,0.0),
			mode: PieceSideType::Unknown,
		}
	}
}

impl PieceSideInfos {
	/// Constructor to init side infos
	pub fn new() -> Self {
		Self {
			left: PieceSideType::Unknown,
			right: PieceSideType::Unknown,
			top: PieceSideType::Unknown,
			bottom: PieceSideType::Unknown,
		}
	}
}

impl Piece {
	fn internal_load(self: &mut Self,img:&image::RgbaImage,back:&image::Rgba<u8>) {
		//extract
		let (x0,y0,w,h) = self.position;

		//take margins to rotate inside
		let (x1,y1) = (EXTRACT_MARGINS*w/100,EXTRACT_MARGINS*h/100);

		//init images
		for pixel in self.image.pixels_mut() {
			*pixel = *back;
		}
		let b = image::Luma([common::MASK_BACKGROUND]);
		for pixel in self.mask.pixels_mut() {
			*pixel = b;
		}

		//copy image & mask
		for y in 0..h {
			for x in 0..w {
				self.image.put_pixel(x1+x,y1+y,*img.get_pixel(x0+x,y0+y));
			}
		}

		//build mask
		let col = image::Luma([common::MASK_PIECE_PIXEL]);
		for y in 0..h {
			for x in 0..w {
				if img.get_pixel(x0+x,y0+y) != back {
					self.mask.put_pixel(x1+x,y1+y,col);
				}
			}
		}
	}

	pub fn load(self: &mut Self,img:&image::RgbaImage,back:&image::Rgba<u8>) {
		//erase
		self.angle = 0;
		self.side_infos = PieceSideInfos::new();
		self.points = PiecePoints::new();
		self.quality = 0;

		//load image
		self.internal_load(img,back);
	}

	/// Constructor of a puzzle piece. It take the global image, the backround color and a square
	/// from which to extract the image and the mask.
	pub fn new(img:&image::RgbaImage,back:&image::Rgba<u8>,square:(u32,u32,u32,u32),id:u32) -> Self {
		//extract
		let (_,_,w,h) = square;

		//take margins to rotate inside
		let (ww,hh) = (w+2*EXTRACT_MARGINS*w/100,h+2*EXTRACT_MARGINS*h/100);

		//create
		let mut cur = Piece {
			id: id,
			position: square,
			image: RgbaImage::new(ww,hh),
			mask: GrayImage::new(ww,hh),
			angle: 0,
			side_infos: PieceSideInfos::new(),
			points: PiecePoints::new(),
			quality: 0,
			faces: [PieceFace::new(),PieceFace::new(),PieceFace::new(),PieceFace::new()],
			matches: [vec!(),vec!(),vec!(),vec!()],
		};

		//load
		cur.internal_load(&img,&back);

		//ret
		cur
	}

	/// Save the image and mask into files for debugging.
	pub fn save(self: &Piece, step: u32, name: &str) {
		//build base name
		let base = format!("step-{:02}-{}-{:05}",step,name,self.id);

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

	fn helper_rotate_coord(value:(u32,u32),rotate:bool,mirror:bool) -> (f32,f32) {
		//extract and convert
		let mut x = value.0 as f32;
		let mut y = value.1 as f32;

		//rotate
		if rotate {
			mem::swap(&mut x,&mut y);
		}

		//mirror
		if mirror {
			x = -x;
		}

		//ret
		(x,y)
	}

	//help
	fn helper_face(top:(u32,u32),middle:(u32,u32),bottom:(u32,u32),rotate:bool,mirror:bool,mode:PieceSideType) -> PieceFace
	{
		PieceFace {
			/*top: Self::helper_rotate_coord(top,rotate,mirror),
			middle: Self::helper_rotate_coord(middle,rotate,mirror),
			bottom: Self::helper_rotate_coord(bottom,rotate,mirror),*/
			top: (top.0 as f32,top.1 as f32),
			middle: (middle.0 as f32,middle.1 as f32),
			bottom: (bottom.0 as f32,bottom.1 as f32),
			mode: mode,
		}
	}

	//extract faces to help matching
	pub fn extract_faces(self: &mut Self) {
		self.faces[TOP] = Self::helper_face(self.points.top_left_corner,self.points.top_shape,self.points.top_right_corner,true,true,self.side_infos.top);
		self.faces[RIGHT] = Self::helper_face(self.points.top_right_corner,self.points.right_shape,self.points.bottom_right_corner,false,false,self.side_infos.right);
		self.faces[BOTTOM] = Self::helper_face(self.points.bottom_left_corner,self.points.bottom_shape,self.points.bottom_right_corner,true,false,self.side_infos.bottom);
		self.faces[LEFT] = Self::helper_face(self.points.top_left_corner,self.points.left_shape,self.points.bottom_left_corner,false,true,self.side_infos.left);
	}
}
