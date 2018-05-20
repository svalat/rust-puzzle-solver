/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// After computing quality mark we can now check and fix pieces with low value

//load external
extern crate image;
extern crate imageproc;
extern crate scoped_pool;

//load std
use std::path::Path;
use std::sync::{Arc,Mutex,RwLock};

//pool
use scoped_pool::Pool;

//load image
use image::GenericImage;

//local
use piece;
use scan;
use common;

pub fn fix_bad_quality(pool: &Pool,all: &Vec<Arc<Mutex<piece::Piece>>>,file:&str,dump:i32)
{
	//reload image as default was modified for search
	let mut img2 = image::open(&Path::new(&file)).unwrap();
	let background = img2.get_pixel(0,0);
	let rgba = Arc::new(RwLock::new(img2.as_mut_rgba8().unwrap()));

	//loop on all and fix those with bad quality rotation
	for pp in all.iter() {
		//extract simple infos in one go and unlock
		let mut quality: u32;
		let mut id: u32;
		let mut position: (u32,u32,u32,u32);
		{
			let mut p = pp.lock().unwrap();
			quality = p.quality;
			id = p.id;
			position = p.position;
		}
		
		//check threashold
		if quality < common::FIX_MIN_QUALITY {
			println!("Redo for {} => {}",id,quality);
			let mut best = Arc::new(Mutex::new((0 as u32,0 as u32)));

			//brut force all angles in parallel
			pool.scoped(|scope| {
                let steps = 90 / common::FIX_ROTATE_ANGLE_RESOLUTION;
				for step in 0..steps {
                    let angle = step * common::FIX_ROTATE_ANGLE_RESOLUTION;
					let rgba = Arc::clone(&rgba);
					let best = Arc::clone(&best);
					scope.execute(move || {
						let r = rgba.read().unwrap();
						let mut test = piece::Piece::new(&r,&background,position,id);
						test.angle = angle;
						scan::scan_piece(&mut test, -1);

						//compare with global progres
						{
							let mut best = best.lock().unwrap();
							if test.quality > best.0 {
								best.0 = test.quality;
								best.1 = angle;
							}
						}
					});
				}
			});

			//apply 
			{
				//rebuild final piece
				let rgba = rgba.read().unwrap();
				let mut f = piece::Piece::new(&rgba,&background,position,id);
				f.angle = best.lock().unwrap().1;
				scan::scan_piece(&mut f, -1);

				//dump
				if dump == 0 || dump == 8 {
					f.save(8,"fix");
				}

				//copy back to override list element
				{
					pp.lock().unwrap().load(&rgba,&background);
				}
			}
		}
	}
}