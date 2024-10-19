use opencv::{
    core::MatTraitConst,
    highgui::{self, WINDOW_AUTOSIZE},
    imgcodecs, Result,
};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <photo_file_path>", args[0]);
        return Ok(());
    }

    let image: opencv::prelude::Mat = imgcodecs::imread(&args[1], imgcodecs::IMREAD_COLOR)?;

    let width = image.size()?.width;
    let height = image.size()?.height;
    println!("image dimensions: {}x{}", width, height);

    highgui::named_window("hello opencv!", WINDOW_AUTOSIZE)?;
    highgui::imshow("hello opencv!", &image)?;
    highgui::wait_key(10000)?;
    Ok(())
}
