/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

//setup colors
pub const MASK_BACKGROUND: u8 = 0u8;
pub const MASK_PIECE_PIXEL: u8 = 127u8;
pub const MASK_SURROUND_RECT: u8 = 255u8;
pub const MASK_HIDDEN_BUMP: u8 = 64u8;
pub const MASK_POINTS_INTREST: u8 = 254u8;
pub const MASK_POINTS_BACK: u8 = 253u8;
pub const MASK_IGNORE_SUPERP: u8 = 16u8;

//quality mark
pub const QUALITY_CORNER_CUT1: f32 = 5.0;
pub const QUALITY_CORNER_CUT2: f32 = 10.0;
pub const QUALITY_BUMP_HOLE_CUT1: f32 = 5.0;
pub const QUALITY_BUMP_HOLE_CUT2: f32 = 10.0;
pub const QUALITY_BUMP_HOLE_CUT3: f32 = 12.0;
pub const QUALITY_BUMP_HOLE_CUT4: f32 = 18.0;

//fix
pub const FIX_MIN_QUALITY: u32 = 16;

//consts
pub const ROTATE_ANGLE_RESOLUTION: u32 = 1;
pub const FIX_ROTATE_ANGLE_RESOLUTION: u32 = 2;

//matching
pub const MATCH_MASK_OFFET: i32 = 3;
pub const MATCH_MASK_OFFSET_STEP:i32 = 3;