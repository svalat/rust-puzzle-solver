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

//modules
mod piece;
mod step1_detect;
mod step3_rotate;

//load std
use std::env;
use std::fs::File;
use std::path::Path;

//load image
use image::GenericImage;

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
	let mut all: Vec<piece::Piece> = Vec::new();

	//find first black pixel
	{
		let rgba8 = img.as_mut_rgba8();
		match rgba8 {
			Some(rgba) => {
				let mut first = (1,1);
				let mut id: u32 = 0;
				while first != (0,0) {
					first = step1_detect::find_first_non_bg_pixel(&rgba,&background);
					println!("First black pixel : {:?}",first);

					//fin square
					let square = step1_detect::find_square_non_bg(&rgba,&background,first);
					println!("Square : {:?}",square);

					//extract into list
					let (_,_,w,h) = square;
					if w*h > 100 {
						all.push(piece::Piece::new(rgba,&background,square,id));
						id = id + 1;
					} else {
						println!("IGNORE, too small !");
					}
					
					//draw for save
					step1_detect::paint_square(rgba,&background,square);
				}
			},
			None => println!("Invalid format ! Expect RGBA8 !")
		}
	}

	//down line
	for p in all.iter_mut() {
		
		println!("------ {:?} ------",p.id);
		let angle = step3_rotate::find_best_rectangle(&p.mask);
		println!("+++> {:?}",angle);
		step3_rotate::draw_best_rectangle(&mut p.mask,angle);

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
