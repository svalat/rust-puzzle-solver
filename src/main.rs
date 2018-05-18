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
extern crate scoped_pool;
extern crate argparse;

//arg parse
use argparse::{ArgumentParser, StoreTrue, Store, List};

//modules
mod piece;
mod step1_detect;
mod step3_rotate;

//load std
use std::fs::File;
use std::path::Path;

//pool
use scoped_pool::Pool;

//load image
use image::GenericImage;

///Main function to run the program.
fn main() {
	//info
	println!("rust-puzzle-solver-0.1.0");

	//parse args
	let mut dump = false;
	let mut threads = 4;
	let mut files:Vec<String> = vec!();
	{
		let mut ap = ArgumentParser::new();
		ap.set_description("Puzzle solver from picture considering a white background bellow the pieces and enough margins arround the pieces.");
        ap.refer(&mut dump)
            .add_option(&["-d", "--dump"], StoreTrue,
				"Dump the picture for every step");
		ap.refer(&mut threads)
			.add_option(&["-t", "--threads"], Store,
				"Number of threads to use (default: 4)");
		ap.refer(&mut files)
            .add_argument("FILES", List,
                "List of files to parse (support only 1 now)");
		ap.parse_args_or_exit();
	}

	//check
	if files.len() != 1 {
		panic!("Invalid number of files, should be exactly one !")
	}
	let file = &files[0];

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

	//create pool
	let pool = Pool::new(threads);

	//down line
	pool.scoped(|scope| {
		for p in all.iter_mut() {
			scope.execute(move || {
				let angle = step3_rotate::find_best_rectangle(&p.mask);
				p.angle = angle;
				println!("+++> {:?} => {:?}",p.id,angle);
				step3_rotate::draw_best_rectangle(&mut p.mask,angle);
			});
		}
	});

	//save list
	if dump {
		pool.scoped(|scope| {
			for p in all.iter() {
				scope.execute(move || {
					p.save(2,"extract");
				});
			}
		});
	}

	//rotate
	pool.scoped(|scope| {
		for p in all.iter_mut() {
			scope.execute(move || {
				println!("Rotate {:?}",p.id);
				p.mask = step3_rotate::do_rotate_gray(& p.mask,p.angle);
				p.image = step3_rotate::do_rotate_rgba(& p.image,p.angle);
			});
		}
	});

	//save list
	if dump {
		pool.scoped(|scope| {
			for p in all.iter() {
				scope.execute(move || {
					p.save(3,"rotate");
				});
			}
		});
	}

	//create output image
	if dump {
		let ref mut fout = File::create("step-1-detect.png").unwrap();
		//write to file
		img.write_to(fout, image::PNG).unwrap();
	}

	}
