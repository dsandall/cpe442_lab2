use opencv::{core::MatTraitConst, highgui, imgcodecs, Result};
use opencv::prelude::MatTrait;

fn main() -> Result<()> {
	let mut image: opencv::prelude::Mat = imgcodecs::imread("lena.jpg", imgcodecs::IMREAD_COLOR)?;

    let mut pixel = image.at_2d_mut::<opencv::core::Vec3b>(0, 0)?;

    // pixel[0] = 0;
    // pixel[1] = 0;
    // pixel[2] = 0;

    let width = image.size()?.width;
    let height = image.size()?.height;
    print!("{}x{}", width, height);
    
	highgui::named_window("hello opencv!", 0)?;
	highgui::imshow("hello opencv!", &image)?;
	highgui::wait_key(10000)?;
	Ok(())
}