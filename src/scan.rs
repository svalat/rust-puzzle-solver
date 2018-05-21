/*****************************************************
             PROJECT  : rust-puzzle-solver
             VERSION  : 0.1.0-dev
             DATE     : 05/2018
             AUTHOR   : Valat Sébastien
             LICENSE  : CeCILL-C
*****************************************************/

/// This provide the main scan methode which is shared by main and step8_fix.

//internal
use piece;
use step3_rotate;
use step4_bump;
use step5_corners;
use step6_hbpoints;
use step7_quality;

//do the scan steps to extract info from images to prepare matching
pub fn scan_piece(p: &mut piece::Piece, dump: i32) {
	//rotate
	println!("Rotate {:?}",p.id);
	p.mask = step3_rotate::do_rotate_gray(& p.mask,p.angle);
	p.image = step3_rotate::do_rotate_rgba(& p.image,p.angle);

	//save
	if dump == 0 || dump == 3 {
		p.save(3,"rotate");
	}

	//remove bumps
	println!("Remove bumps");
	p.side_infos = step4_bump::remove_bumps(&mut p.mask);

	//save
	if dump == 0 || dump == 4 {
		p.save(4,"remove-bump");
	}

	//extract points
	println!("Extract corners");
	p.points = step5_corners::extract_piece_points(&p.mask,&p.side_infos);
	step5_corners::draw_corners(&mut p.mask,&p.points);

	//save
	if dump == 0 || dump == 5 {
		p.save(5,"corners");
	}

	//extract points
	println!("Extract holes/bump points");
	step6_hbpoints::extract_piece_points(&p.mask,&mut p.points,&p.side_infos);
	step6_hbpoints::draw_corners(&mut p.mask,&p.points);

	//save
	if dump == 0 || dump == 6 {
		p.save(6,"hbpoints");
	}

	//check quality
	p.quality = step7_quality::calc_quality_mark(&p,dump);
	println!("Quality = {}",p.quality);

	//extract face numbers
	p.extract_faces();
}
