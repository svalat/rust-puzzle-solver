/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// From experiments sometimes we miss rotate the pieces by choosing the wrong angle. This append
/// most of the time with peices with 3 holes and 1 bump. In order to fix those pieces we tried
/// to make a quality check based on properties expected for the 8 points extracted from the picture
/// (4 corners and 4 holes/bump).
/// From this we can detect badly analysed pieces and maybe make a brut force search with all rotation
/// angle to find one with better quality.
/// In order to proceed we check various constrain and attribute a quality mark.

//load external
extern crate image;

//std
use std::fs::File;
use std::io::Write;

//internal
use step5_corners;
use piece::{Piece,PiecePoints};

//cut
const CORNER_CUT1: f32 = 5.0;
const CORNER_CUT2: f32 = 10.0;
const BUMP_HOLE_CUT1: f32 = 5.0;
const BUMP_HOLE_CUT2: f32 = 10.0;

fn apply_cut(value: f32, cut1:f32, cut2:f32) -> u32 {
    let mut mark = 0;
    if value <= cut1 {
        mark += 1;
    }
    if value <= cut2 {
        mark += 1;
    }
    mark
}

fn check_holes_bumps(p: &PiecePoints, size: (u32,u32), db: &mut Vec<f32>) -> u32 {
    //vars
    let (w,h) = size;

    //calc dist
    let vert = (p.top_shape.0 as i32 - p.bottom_shape.0 as i32).abs();
    let horiz = (p.left_shape.1 as i32 - p.right_shape.1 as i32).abs();
    
    //compute %
    let vert = (vert * vert / w as i32) as f32;
    let horiz = (100 * horiz / h as i32) as f32;
    
    //add to DB
    db.push(vert);
    db.push(horiz);

    //apply cut nd count quality points
    let mut mark = 0;
    mark += apply_cut(vert,BUMP_HOLE_CUT1,BUMP_HOLE_CUT2);
    mark += apply_cut(horiz,BUMP_HOLE_CUT1,BUMP_HOLE_CUT2);

    //ret
    mark
}

fn check_corners(p: &PiecePoints, size: (u32,u32), db: &mut Vec<f32>) -> u32 {
    //vars
    let (w,h) = size;

    //calc dist
    let left = (p.top_left_corner.0 as i32 - p.bottom_left_corner.0 as i32).abs();
    let right = (p.top_right_corner.0 as i32 - p.bottom_right_corner.0 as i32).abs();
    let top = (p.top_left_corner.1 as i32 - p.top_right_corner.1 as i32).abs();
    let bottom = (p.bottom_left_corner.1 as i32 - p.bottom_right_corner.1 as i32).abs();

    //compute %
    let left = (100 * left / w as i32) as f32;
    let right = (100 * right / w as i32) as f32;
    let top = (100 * top / h as i32) as f32;
    let bottom = (100 * bottom / h as i32) as f32;

    //add to DB
    db.push(left);
    db.push(right);
    db.push(top);
    db.push(bottom);

    //apply cut nd count quality points
    let mut mark = 0;
    mark += apply_cut(left,CORNER_CUT1,CORNER_CUT2);
    mark += apply_cut(right,CORNER_CUT1,CORNER_CUT2);
    mark += apply_cut(top,CORNER_CUT1,CORNER_CUT2);
    mark += apply_cut(bottom,CORNER_CUT1,CORNER_CUT2);

    //ret
    mark
}

//main entry point
pub fn calc_quality_mark(piece: &Piece, dump: i32) -> u32 {
    //vars
    let rect = step5_corners::extract_surrounding_rect(&piece.mask);
    let (w,h) = (rect.2 - rect.0,rect.3 - rect.1);
    let size = (w,h);
    let mut mark = 0;
    let mut db: Vec<f32> = vec!();

    //first is if
    db.push(piece.id as f32);

    //apply
    mark += check_corners(&piece.points,size,&mut db);
    mark += check_holes_bumps(&piece.points,size,&mut db);

    //add mark
    db.push(mark as f32);

    //dump db into file
    if dump == 0 || dump == 7 {
        let base = format!("step-7-quality-{:05}.txt",piece.id);
        let mut file = File::create(base).unwrap();
        file.write_fmt(format_args!("{:?}\n",db)).unwrap();
    }

    mark
}