use opencv::{core::MatTraitConst, highgui::{self, WINDOW_AUTOSIZE, WINDOW_NORMAL}, imgcodecs, Result};
use std::env;
// use opencv::prelude::MatTrait;

fn main() -> Result<()> {

    let args: Vec<String> = env::args().collect();

	let image: opencv::prelude::Mat = imgcodecs::imread(&args[1], imgcodecs::IMREAD_COLOR)?;

    // let mut pixel = image.at_2d_mut::<opencv::core::Vec3b>(0, 0)?;

    // pixel[0] = 0;
    // pixel[1] = 0;
    // pixel[2] = 0;

    let width = image.size()?.width;
    let height = image.size()?.height;
    println!("{}x{}", width, height);
    
	highgui::named_window("hello opencv!", WINDOW_AUTOSIZE)?;
	highgui::imshow("hello opencv!", &image)?;
	highgui::wait_key(10000)?;
	Ok(())
}