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
extern crate ndarray;
extern crate num_traits;

//modules
mod piece;
mod step1_detect;
mod step3_rotate;
mod step4_bump;
mod step5_corners;
mod step6_hbpoints;
mod step7_quality;
mod step8_fix;
mod step9_cleanup;
mod step10_matching;
mod step11_build;
mod step12_draw;
mod common;
mod scan;

//arg parse
use argparse::{ArgumentParser, Store, List};

//load std
use std::fs::File;
use std::path::Path;
use std::sync::{Arc,Mutex};

//pool
use scoped_pool::Pool;

//load image
use image::GenericImage;

//interal
use piece::PieceMatch;

///Main function to run the program.
fn main() {
	//info
	println!("rust-puzzle-solver-0.1.0");

	//parse args
	let mut dump:i32 = -1;
	let mut threads = 4;
	let mut files:Vec<String> = vec!();
	{
		let mut ap = ArgumentParser::new();
		ap.set_description("Puzzle solver from picture considering a white background bellow the pieces and enough margins arround the pieces.");
        ap.refer(&mut dump)
            .add_option(&["-d", "--dump"], Store,
				"Dump the picture for given step, use 0 for all");
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
	let mut all: piece::PieceVec = Vec::new();

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
					if w*h > 600 {
						all.push(Arc::new(Mutex::new(piece::Piece::new(rgba,&background,square,id))));
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

	//create output image
	if dump == 0 || dump == 1 {
		let ref mut fout = File::create("step-1-detect.png").unwrap();
		//write to file
		img.write_to(fout, image::PNG).unwrap();
	}

	//create pool
	let pool = Pool::new(threads);

	//prepare pieces
	pool.scoped(|scope| {
		for pp in all.iter_mut() {
			scope.execute(move || {
				let mut p = pp.lock().unwrap();
				let angle = step3_rotate::find_best_rectangle(&p.mask);
				p.angle = angle;
				println!("=============> {:?} => {:?} <==============",p.id,angle);
				step3_rotate::draw_best_rectangle(&mut p.mask,angle);

				//save
				if dump == 0 || dump == 2 {
					p.save(2,"extract");
				}

				//do all jobs
				scan::scan_piece(&mut p, dump);
			});
		}
	});

	//loop on invalid to fix them
	println!("===============================");
	step8_fix::fix_bad_quality(&pool,&all,file,dump);

	//cleanup
	pool.scoped(|scope| {
		for pp in all.iter_mut() {
			scope.execute(move || {
				let mut p = pp.lock().unwrap();
				step9_cleanup::clear_debug_markers(&mut p.mask);
				if dump == 0 || dump == 9 {
					p.save(9,"cleanup");
				}
			});
		}
	});

	//do matchin
	//step10_matching::compute_matching(&mut all,dump);

	//let links
	{
		let mut p0 = all[0].lock().unwrap();
		let mut p1 = all[1].lock().unwrap();
		let mut p2 = all[2].lock().unwrap();
		let mut p3 = all[3].lock().unwrap();
		let mut p4 = all[4].lock().unwrap();
		let mut p5 = all[5].lock().unwrap();
		let mut p6 = all[6].lock().unwrap();
		let mut p7 = all[7].lock().unwrap();
		let mut p8 = all[8].lock().unwrap();

		p5.matches[0].push(PieceMatch{piece:7,side:3,angle:0.0,distance:0.0});
		p7.matches[0].push(PieceMatch{piece:3,side:2,angle:0.0,distance:0.0});
		p4.matches[1].push(PieceMatch{piece:8,side:0,angle:0.0,distance:0.0});
		p8.matches[1].push(PieceMatch{piece:6,side:3,angle:0.0,distance:0.0});
		p7.matches[1].push(PieceMatch{piece:4,side:2,angle:0.0,distance:0.0});
		p3.matches[1].push(PieceMatch{piece:2,side:2,angle:0.0,distance:0.0});
		p4.matches[3].push(PieceMatch{piece:2,side:1,angle:0.0,distance:0.0});
		p2.matches[3].push(PieceMatch{piece:0,side:2,angle:0.0,distance:0.0});
		p2.matches[0].push(PieceMatch{piece:1,side:2,angle:0.0,distance:0.0});

		p7.matches[3].push(PieceMatch{piece:5,side:0,angle:0.0,distance:0.0});
		p3.matches[2].push(PieceMatch{piece:7,side:0,angle:0.0,distance:0.0});
		p8.matches[0].push(PieceMatch{piece:4,side:1,angle:0.0,distance:0.0});
		p6.matches[3].push(PieceMatch{piece:8,side:1,angle:0.0,distance:0.0});
		p4.matches[2].push(PieceMatch{piece:7,side:1,angle:0.0,distance:0.0});
		p2.matches[2].push(PieceMatch{piece:3,side:1,angle:0.0,distance:0.0});
		p2.matches[1].push(PieceMatch{piece:4,side:3,angle:0.0,distance:0.0});
		p0.matches[2].push(PieceMatch{piece:2,side:3,angle:0.0,distance:0.0});
		p1.matches[2].push(PieceMatch{piece:2,side:0,angle:0.0,distance:0.0});
	}

	//build
	let sol = step11_build::build_solution(&mut all,dump);

	//draw
	step12_draw::draw_solution(&sol,&all,"step-12-solu-0001.png");
}
