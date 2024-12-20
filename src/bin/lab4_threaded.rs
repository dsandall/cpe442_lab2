use opencv::{
    boxed_ref::BoxedRef,
    core::{Mat, Rect, CV_8UC1},
    highgui::{self, WINDOW_AUTOSIZE},
    prelude::*,
    videoio, Result,
};
use rayon::prelude::*;
use std::env;
use std::time::Instant;

const NUM_THREADS: usize = 4;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <video_file_path>", args[0]);
        return Ok(());
    }

    // Open the video file
    let mut video = videoio::VideoCapture::from_file(&args[1], videoio::CAP_ANY)?;
    if !video.is_opened()? {
        panic!("Error: Couldn't open video file.");
    }

    // Create a window to display frames
    highgui::named_window("Video Frame", WINDOW_AUTOSIZE)?;
    highgui::named_window("Video Frame2", WINDOW_AUTOSIZE)?;

    let mut total_sobel_time = std::time::Duration::new(0, 0);
    let mut frame_count = 0;

    loop {
        // Read the next frame
        let mut frame = Mat::default();
        if !video.read(&mut frame)? {
            println!("Video processing finished.");
            break;
        } else if frame.empty() {
            println!("Empty frame detected. Video might have ended.");
            break;
        }

        // Start timing for Sobel filter
        let start_sobel = Instant::now();

        // Do the actual frame stuff
        let combined_frame = do_frame(&frame)?;

        // Handle timing tracking
        let sobel_duration = start_sobel.elapsed();
        total_sobel_time += sobel_duration;
        frame_count += 1;

        // (Optional) Save or display the combined frame
        // opencv::imgcodecs::imwrite("./YAHOO.jpg", &combined_frame, &opencv::core::Vector::from_slice(&[0]))?;

        // // Display the frames in the windows
        highgui::imshow("Video Frame", &combined_frame)?;
        highgui::imshow("Video Frame2", &frame)?;

        // Wait for 30ms between frames
        if highgui::wait_key(1)? == 27 {
            // Exit if the 'ESC' key is pressed
            println!("ESC key pressed. Exiting...");
            break;
        }

        // Every 50 frames, calculate and print averages
        if frame_count % 50 == 0 {
            let avg_sobel_time = total_sobel_time / frame_count;
            println!(
                "Averages after {} frames: Sobel: {:?}",
                frame_count, avg_sobel_time
            );
        }
    }

    Ok(())
}

fn do_frame(frame: &Mat) -> Result<Mat> {
    // Calculate the height for each smaller matrix
    let split_height = frame.rows() / NUM_THREADS as i32;

    // Create the smaller matrices with the specified overlaps
    let mat1 = Mat::roi(frame, Rect::new(0, 0, frame.cols(), split_height + 1))?;
    let mat2 = Mat::roi(
        frame,
        Rect::new(0, split_height - 1, frame.cols(), split_height + 2),
    )?;
    let mat3 = Mat::roi(
        frame,
        Rect::new(0, split_height * 2 - 1, frame.cols(), split_height + 2),
    )?;
    let mat4 = Mat::roi(
        frame,
        Rect::new(0, split_height * 3 - 1, frame.cols(), split_height + 1),
    )?;

    //move these to parallel
    let mats = vec![mat1, mat2, mat3, mat4];
    let sobel_results = do_sobel_parallel(&mats)?;
    //end parallel

    // Trim the results
    let mat1_trimmed = Mat::roi(
        &sobel_results[0],
        Rect::new(
            1,
            1,
            sobel_results[0].cols() - 2,
            sobel_results[0].rows() - 2,
        ),
    )?;
    let mat2_trimmed = Mat::roi(
        &sobel_results[1],
        Rect::new(
            1,
            1,
            sobel_results[1].cols() - 2,
            sobel_results[1].rows() - 2,
        ),
    )?;
    let mat3_trimmed = Mat::roi(
        &sobel_results[2],
        Rect::new(
            1,
            1,
            sobel_results[2].cols() - 2,
            sobel_results[2].rows() - 2,
        ),
    )?;
    let mat4_trimmed = Mat::roi(
        &sobel_results[3],
        Rect::new(
            1,
            1,
            sobel_results[3].cols() - 2,
            sobel_results[3].rows() - 1,
        ),
    )?;

    // Create a new Mat for the combined result
    let combined_height =
        mat1_trimmed.rows() + mat2_trimmed.rows() + mat3_trimmed.rows() + mat4_trimmed.rows(); // Total height
    let mut combined_frame =
        unsafe { Mat::new_rows_cols(combined_height, mat1_trimmed.cols(), CV_8UC1) }?; // Create an empty matrix of the appropriate size

    // Copy the data from each matrix into the combined frame
    let mut current_row = 0;

    for mat in &[mat1_trimmed, mat2_trimmed, mat3_trimmed, mat4_trimmed] {
        // Create a ROI for the current position in the combined frame
        let mut roi = Mat::roi_mut(
            &mut combined_frame,
            Rect::new(0, current_row, mat.cols(), mat.rows()),
        )?;

        // Copy the data
        mat.copy_to(&mut roi)?;

        current_row += mat.rows(); // Move to the next position
    }

    Ok(combined_frame)
}

// Process Sobel in parallel
fn do_sobel_parallel(mats: &[BoxedRef<'_, Mat>]) -> Result<Vec<Mat>> {
    let results: Vec<Mat> = mats
        .par_iter()
        .map(|mat| to442_sobel(&to442_grayscale(mat).unwrap()).unwrap())
        .collect();

    // // Sequential implementation (still splits the frame)
    // let results = vec![to442_sobel(&to442_grayscale(&mats[0]).unwrap()).unwrap(),
    // to442_sobel(&to442_grayscale(&mats[1]).unwrap()).unwrap(),
    // to442_sobel(&to442_grayscale(&mats[2]).unwrap()).unwrap(),
    // to442_sobel(&to442_grayscale(&mats[3]).unwrap()).unwrap()];

    Ok(results)
}

fn to442_grayscale(frame: &opencv::mod_prelude::BoxedRef<'_, Mat>) -> Result<Mat> {
    let mut output: Mat =
        unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };

    // Convert the frame reference to a mutable slice of `u8`
    let data_slice: &[u8] = unsafe {
        std::slice::from_raw_parts(frame.data(), (frame.rows() * frame.cols() * 3) as usize)
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
        *output
            .at_2d_mut::<u8>(i / frame.cols(), i % frame.cols())
            .unwrap() = gray_value;

        // println!("Grayscale value at pixel {}: {}", i, gray_value);

        i += 1;
    });

    Ok(output)
}

fn to442_sobel(frame: &Mat) -> Result<Mat> {
    let mut output: Mat =
        unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };

    let gx: [[i32; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
    let gy: [[i32; 3]; 3] = [[1, 2, 1], [0, 0, 0], [-1, -2, -1]];

    for y in 1..(frame.rows() - 1) {
        for x in 1..(frame.cols() - 1) {
            let (sum_x, sum_y) = (0..3)
                .flat_map(|ky| {
                    (0..3).map(move |kx| {
                        let pixel: i32 =
                            (*frame.at_2d::<u8>(y + ky - 1, x + kx - 1).unwrap()).into();
                        (
                            pixel * gx[ky as usize][kx as usize],
                            pixel * gy[ky as usize][kx as usize],
                        )
                    })
                })
                .fold((0i32, 0i32), |(acc_x, acc_y), (dx, dy)| {
                    (acc_x + dx, acc_y + dy)
                });

            let magnitude = (sum_x.abs() + sum_y.abs()).min(255) as u8;

            *(output.at_2d_mut::<u8>(y, x)?) = magnitude;
        }
    }

    Ok(output)
}
