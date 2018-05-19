/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

//load external
extern crate image;

//consts
const BUMP_SEGMENTS: u32 = 20;

/// Compute segment start and size
fn build_segment_info(size:u32, seg_id:u32) -> (u32,u32)
{
    let start = (seg_id * size) / BUMP_SEGMENTS;
    let end = ((seg_id+1) * size) / BUMP_SEGMENTS;
    return (start,end-start);
}

/// Do operation on each pixel of a segment
fn do_op_on_segment<P>(start:(u32,u32),step:(u32,u32),size:u32,seg_id:u32,mut f:P) 
where
P: FnMut(u32,u32)
{
    //get segment
    let (seg_start,seg_size) = build_segment_info(size,seg_id);

    //compute start
    let mut cur = (start.0 + step.0 * seg_start,start.1 + step.1 * seg_start);

    //loop
    for _ in 0..seg_size {
        //to
        f(cur.0,cur.1);

        //move
        cur.0 += step.0;
        cur.1 += step.1;
    }
}

/// Split the given line in segments and count the number of active pixel in the given segment
fn count_pixel_on_segment(img: &image::GrayImage,start:(u32,u32),step:(u32,u32),size:u32,seg_id:u32) -> u32
{
    //counter & color
    let mut cnt = 0;
    let color = image::Luma([128u8]);

    //run over pixel of segment
    do_op_on_segment(start,step,size,seg_id,|x:u32,y:u32| {
        if *img.get_pixel(x,y) == color {
            cnt += 1;
        }
    });

    //ret
    cnt
}

/// Check if a line is a bump and mask pixel if ture
fn check_and_mask_bump(img: &mut image::GrayImage,start:(u32,u32),step:(u32,u32),size:u32) -> bool {
    //storage
    let mut segs: Vec<u32> = vec!();

    //fill
    for i in 0..BUMP_SEGMENTS {
        segs.push(count_pixel_on_segment(img,start,step,size,i));
    }

    //get first segment
    let mut ret = false;
    let mut cnt_empty_start = 0;
    let mut cur: usize = 0;
    while cur < BUMP_SEGMENTS as usize && segs[cur] == 0 {
        cnt_empty_start += 1;
        cur += 1;
    }

    //now search non empty (bump)
    let mut cnt_bump = 0;
    while cur < BUMP_SEGMENTS as usize && segs[cur] > 0 {
        cnt_bump += 1;
        cur += 1;
    }


    //now search end empty
    let mut cnt_empty_end = 0;
    while cur < BUMP_SEGMENTS as usize && segs[cur] == 0 {
        cnt_empty_end += 1;
        cur += 1;
    }

    //println!("Buma param : {}, {}, {} => {} == {}",cnt_empty_start,cnt_bump,cnt_empty_end, cur, BUMP_SEGMENTS);

    //check if is bump
    let threashold = BUMP_SEGMENTS / 6;
    if cur == BUMP_SEGMENTS as usize && cnt_empty_start >= threashold && cnt_bump >= 1 && cnt_bump < BUMP_SEGMENTS / 2 && cnt_empty_end >= threashold {
        //mark as bump
        ret = true;

        //erase pixel
        let color_exp = image::Luma([128u8]);
        let color_set = image::Luma([64u8]);
        for i in 0..size {
            let x = start.0 + step.0 * i;
            let y = start.1 + step.1 * i;
            let mut pixel = img.get_pixel_mut(x,y);
            if *pixel == color_exp {
                *pixel = color_set;
            }
        }
    }

    ret
}

/// Remove bump on one side
fn remove_bump_one_side(img: &mut image::GrayImage,start:(u32,u32),step:(u32,u32),step_op:(i32,i32),size:u32,size_op:u32) -> bool {
    //find first none empty line
    let color = image::Luma([128u8]);
    let mut pos = 0;
    for j in 0..size_op {
        let start = ((start.0 as i32+step_op.0*j as i32) as u32,(start.1 as i32+step_op.1*j as i32) as u32);
        let mut has_pixel = false;
        for i in 0..size {
            if *img.get_pixel(start.0+step.0*i,start.1+step.1*i) == color {
                pos = j;
                has_pixel = true;
                break;
            }
        }
        if has_pixel {
            break;
        }
    }

    //println!("Bump {:?} -> {:?} -> {:?}, pos = {:?}",start,step,size,pos);
    
    //loop until not bump anymore
    let mut has_bump = false;
    for i in pos..size_op {
        let has = check_and_mask_bump(img,((start.0 as i32+i as i32*step_op.0) as u32,(start.1 as i32+i as i32*step_op.1) as u32),step,size);
        if has {
            //println!("Has bump");
            has_bump = true;
        } else {
            break;
        }
    }

    has_bump
}

/// Remove bump on 4 sides
pub fn remove_bumps(img: &mut image::GrayImage) {
    let (w,h) = img.dimensions();

    remove_bump_one_side(img,(0,0),(1,0),(0,1),w,h);
    remove_bump_one_side(img,(0,0),(0,1),(1,0),h,w);
    remove_bump_one_side(img,(0,h-1),(1,0),(0,-1),w,h);
    remove_bump_one_side(img,(w-1,0),(0,1),(-1,0),h,w);
}
