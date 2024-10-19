use opencv::{
    core::{self, Mat},
    highgui::{self, WINDOW_AUTOSIZE},
    videoio,
    prelude::*,
    Result,
};

use opencv::core::Vec3b;

use std::env;

use opencv::core::CV_8UC1;

fn dumb_copy(frame: &mut Mat) -> Result<Mat> {
    // Create an output matrix of type CV_8UC1 for grayscale
    let mut output: Mat = unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };

    // Get the total size of the image data (assuming the input is in BGR format)
    let total_size = (frame.rows() * frame.cols() * 3) as usize;

    // Convert the raw pointer to a mutable slice of `u8`
    let data_slice: &mut [u8] = unsafe {
        std::slice::from_raw_parts_mut(frame.data_mut(), total_size)
    };

    // Use chunks_exact(3) to process the image data in groups of 3 (BGR channels)
    let mut i = 0;
    data_slice.chunks_exact(3).for_each(|pixel| {
        let b = pixel[0] as f32; // Blue channel
        let g = pixel[1] as f32; // Green channel
        let r = pixel[2] as f32; // Red channel

        // Apply the grayscale formula
        let gray_value = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u8;

        // Set the pixel value in the output matrix
        *output.at_2d_mut::<u8>(i / frame.cols(), i % frame.cols()).unwrap() = gray_value;

        // Optional: Print the grayscale value
        // println!("Grayscale value at pixel {}: {}", i, gray_value);

        i += 1;
    });

    Ok(output)
}


fn to442_grayscale(frame: &mut Mat) -> Result<Mat> {
    // Get the total size of the image data
    let total_size = (frame.rows() * frame.cols() * 3) as usize;

    // Convert the raw pointer to a mutable slice of `u8`
    let data_slice: &mut [u8] = unsafe {
        std::slice::from_raw_parts_mut(frame.data_mut(), total_size)
    };

    let mut output : Mat = unsafe{opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)?}; // 16-bit unsigned single-channel

    // Use chunks_exact_mut(3) to process the image data in groups of 3 (BGR channels)
    let mut i = 0;
    data_slice.chunks_exact_mut(3).for_each(|pixel| {
        let b = pixel[0] as f32; // Blue channel
        let g = pixel[1] as f32; // Green channel
        let r = pixel[2] as f32; // Red channel

        // Apply the grayscale formula
        let gray_value_u16 = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u16;
        let gray_value: u8 = std::cmp::min(gray_value_u16 as u8, 255);

        // Set the pixel to the grayscale value
        *(output.at_2d_mut(i%frame.rows(), i/frame.cols()).unwrap()) = gray_value;

        println!("{}", output.at_2d_mut::<u8>(i%frame.rows(), i/frame.cols()).unwrap());

        i += 1;
    });

    Ok(output)
}

use opencv::core::{CV_16UC1, NORM_MINMAX};
fn convert_u16_to_8bit(input: &Mat) -> Result<Mat> {

    // Create an empty output matrix with the same rows and cols, but type CV_8UC1
    let mut output = Mat::default();
    
    // Normalize the u16 matrix to the range [0, 255] and convert it to 8-bit
    opencv::core::normalize(&input, &mut output, 0.0, 255.0, NORM_MINMAX, CV_8UC1, &Mat::default())?;

    Ok(output)
}

// fn to442_sobel(frame: &Mat) -> Result<Mat> {
//     let (rows, cols) = (frame.rows(), frame.cols());

//     let mut output: Mat = unsafe { opencv::core::Mat::new_rows_cols(rows, cols, CV_16UC1)? };

//     let gx: [[i32; 3]; 3] = [[-1, 0, 1],
//                              [-2, 0, 2],
//                              [-1, 0, 1]];

//     let gy: [[i32; 3]; 3] = [[1, 2, 1],
//                              [0, 0, 0],
//                              [-1, -2, -1]];

//     for y in 1..(rows - 1) {
//         for x in 1..(cols - 1) {
//             let (sum_x, sum_y) = (0..3).flat_map(|ky| {
//                 (0..3).map(move |kx| {
//                     let pixel = frame.at_2d::<u16>(y + ky - 1, x + kx - 1).unwrap();
//                     let pixel_value: i32 = pixel.into(); // Explicitly convert to i32
//                     (
//                         pixel_value * gx[ky as usize][kx as usize],
//                         pixel_value * gy[ky as usize][kx as usize],
//                     )
//                 })
//             }).fold((0i32, 0i32), |(acc_x, acc_y), (dx, dy)| (acc_x + dx, acc_y + dy)); // Specify i32 for sum_x and sum_y

//             let magnitude = ((sum_x.abs() + sum_y.abs()).min(255)) as u16;

//             *(output.at_2d_mut::<u16>(y, x)?) = magnitude;
//         }
//     }

//     Ok(output)
// }



fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <video_file_path>", args[0]);
        return Ok(());
    }

    // // Initialize PAPI
    // papi_bindings::initialize(false).unwrap();
    // // papi_bindings::events_set::EventsSet::new(counters);

    // Open the video file (pass the path to the video file as an argument)
    let mut video = videoio::VideoCapture::from_file(&args[1], videoio::CAP_ANY)?;
    if !video.is_opened()? {
        panic!("Error: Couldn't open video file.");
    }

    // Create a window to display frames
    highgui::named_window("Video Frame", WINDOW_AUTOSIZE)?;
    highgui::named_window("Video Frame2", WINDOW_AUTOSIZE)?;


    loop {
        // Create a Mat to hold the frame
        let mut frame = Mat::default();

        // Read the next frame
        if !video.read(&mut frame)? {
            println!("Video processing finished.");
            break;
        }

        // Check if the frame is empty (video end)
        if frame.empty() {
            println!("Empty frame detected. Video might have ended.");
            break;
        }

        // Convert the frame to grayscale
        let intermediary = dumb_copy(&mut frame)?;
        // let grayyy = convert_u16_to_8bit(&intermediary)?;
        // let frame_sobel = to442_sobel(&intermediary).unwrap();

        // Display the grayscale frame in the window
        highgui::imshow("Video Frame", &intermediary)?;
        highgui::imshow("Video Frame2", &frame)?;

        // Wait for 30ms between frames (this sets the frame rate, e.g., ~33 fps)
        if highgui::wait_key(30)? == 27 {
            // Exit if the 'ESC' key is pressed
            println!("ESC key pressed. Exiting...");
            break;
        }
    }

    Ok(())
}
