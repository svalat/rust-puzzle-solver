/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// In all the previous steps we drawed some debug infos on the mask
/// In order to ease the maching function we first remove all those markers

//packages
extern crate image;

//local
use common::*;

pub fn clear_debug_markers(img: &mut image::GrayImage) {
    //build colors
    let back = image::Luma([MASK_BACKGROUND]);
    let surrounding_rect = image::Luma([MASK_SURROUND_RECT]);
    let hidden_bump = image::Luma([MASK_HIDDEN_BUMP]);
    let interest = image::Luma([MASK_PIECE_PIXEL]);
    let point_back = image::Luma([MASK_POINTS_BACK]);
    let point_interes = image::Luma([MASK_POINTS_INTREST]);

    //apply on all pixels
    for pixel in img.pixels_mut() {
        if *pixel == surrounding_rect || *pixel == point_back {
            *pixel = back;
        }
        if *pixel == hidden_bump || *pixel == point_interes {
            *pixel = interest;
        }
    }
}