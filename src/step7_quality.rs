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
use piece::{Piece,PiecePoints,PieceSideType,PieceSideInfos};
use common::*;

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

fn apply_cut_sup(value: f32, cut1:f32, cut2:f32) -> u32 {
    let mut mark = 0;
    if value >= cut1 {
        mark += 1;
    }
    if value >= cut2 {
        mark += 1;
    }
    mark
}

fn fix_bump_hole_sign(value:f32,t: &PieceSideType,sign:f32) -> f32 {
    let mut ret = value;
    match t {
        PieceSideType::Bump => {},
        PieceSideType::Hole => ret *= -1.0,
        PieceSideType::Unknown => panic!("Should not append here !"),
    }

    ret * sign
}

fn check_holes_bumps_topo(p: &PiecePoints, t: &PieceSideInfos, size: (u32,u32), db: &mut Vec<f32>) -> u32 {
    //extract
    let (w,h) = (size.0 as f32,size.1 as f32);
    let mut mark = 0;

    //extract
    let left1 = 100.0 * (p.left_shape.0 as f32 - p.top_left_corner.0 as f32 ) / w;
    let left2 = 100.0 * (p.left_shape.0 as f32  - p.bottom_left_corner.0 as f32 ) / w;
    let right1 = 100.0 * (p.right_shape.0 as f32  - p.top_right_corner.0 as f32 ) / w;
    let right2 = 100.0 * (p.right_shape.0 as f32  - p.bottom_right_corner.0 as f32 ) / w;
    let top1 = 100.0 * (p.top_shape.1 as f32  - p.top_left_corner.1 as f32 ) / h;
    let top2 = 100.0 * (p.top_shape.1 as f32  - p.top_right_corner.1 as f32 ) / h;
    let bottom1 = 100.0 * (p.bottom_shape.1 as f32  - p.bottom_left_corner.1 as f32 ) / h;
    let bottom2 = 100.0 * (p.bottom_shape.1 as f32  - p.bottom_right_corner.1 as f32 ) / h;

    //fix sign
    let left1 = fix_bump_hole_sign(left1,&t.left,1.0);
    let left2 = fix_bump_hole_sign(left2,&t.left,1.0);
    let right1 = fix_bump_hole_sign(right1,&t.right,-1.0);
    let right2 = fix_bump_hole_sign(right2,&t.right,-1.0);
    let top1 = fix_bump_hole_sign(top1,&t.top,-1.0);
    let top2 = fix_bump_hole_sign(top2,&t.top,-1.0);
    let bottom1 = fix_bump_hole_sign(bottom1,&t.bottom,1.0);
    let bottom2 = fix_bump_hole_sign(bottom2,&t.bottom,1.0);

    //fill db
    db.push(left1);
    db.push(left2);
    db.push(right1);
    db.push(right2);
    db.push(top1);
    db.push(top2);
    db.push(bottom1);
    db.push(bottom2);

    //mark
    mark += apply_cut_sup(left1,QUALITY_BUMP_HOLE_CUT3,QUALITY_BUMP_HOLE_CUT4);
    mark += apply_cut_sup(left2,QUALITY_BUMP_HOLE_CUT3,QUALITY_BUMP_HOLE_CUT4);
    mark += apply_cut_sup(right1,QUALITY_BUMP_HOLE_CUT3,QUALITY_BUMP_HOLE_CUT4);
    mark += apply_cut_sup(right2,QUALITY_BUMP_HOLE_CUT3,QUALITY_BUMP_HOLE_CUT4);
    mark += apply_cut_sup(top1,QUALITY_BUMP_HOLE_CUT3,QUALITY_BUMP_HOLE_CUT4);
    mark += apply_cut_sup(top2,QUALITY_BUMP_HOLE_CUT3,QUALITY_BUMP_HOLE_CUT4);
    mark += apply_cut_sup(bottom1,QUALITY_BUMP_HOLE_CUT3,QUALITY_BUMP_HOLE_CUT4);
    mark += apply_cut_sup(bottom2,QUALITY_BUMP_HOLE_CUT3,QUALITY_BUMP_HOLE_CUT4);

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
    mark += apply_cut(vert,QUALITY_BUMP_HOLE_CUT1,QUALITY_BUMP_HOLE_CUT2);
    mark += apply_cut(horiz,QUALITY_BUMP_HOLE_CUT1,QUALITY_BUMP_HOLE_CUT2);

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
    mark += apply_cut(left,QUALITY_CORNER_CUT1,QUALITY_CORNER_CUT2);
    mark += apply_cut(right,QUALITY_CORNER_CUT1,QUALITY_CORNER_CUT2);
    mark += apply_cut(top,QUALITY_CORNER_CUT1,QUALITY_CORNER_CUT2);
    mark += apply_cut(bottom,QUALITY_CORNER_CUT1,QUALITY_CORNER_CUT2);

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
    mark += check_holes_bumps_topo(&piece.points,&piece.side_infos,size,&mut db);

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