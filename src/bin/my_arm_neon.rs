#![cfg(target_arch = "aarch64")]

use std::arch::aarch64::{vadd_s8, vaddq_u32, vdupq_n_f32, vgetq_lane_f32, vld1q_f32, vld1q_u8, vmlaq_f32, vreinterpretq_u32_u16, vst1_u8};
use opencv::{
    boxed_ref::BoxedRef, core::{Mat, CV_8UC1}, prelude::*, Result
};


fn to442_grayscale(frame: &opencv::mod_prelude::BoxedRef<'_, Mat>) -> Result<Mat> {

    let mut output: Mat = unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };

    // Convert the frame reference to a mutable slice of `u8`
    let data_slice: &[u8] = unsafe { std::slice::from_raw_parts(frame.data(), (frame.rows() * frame.cols() * 3) as usize) };

    // Use chunks_exact(3) to process the image data in groups of 3 (BGR channels)
    let mut i = 0;
    data_slice.chunks_exact(3*4).for_each(|pixel| {
        // let b = data[0] as f32; // Blue channel
        // let g = data[1] as f32; // Green channel
        // let r = data[2] as f32; // Red channel

        // // Apply the grayscale formula
        // let gray_value = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u8;

        // // Set the pixel value in the output matrix
        // *output
        //     .at_2d_mut::<u8>(i / frame.cols(), i % frame.cols())
        //     .unwrap() = gray_value;

        // // println!("Grayscale value at pixel {}: {}", i, gray_value);

        i += 4;
    });

    Ok(output)
}


use std::arch::aarch64::{vreinterpretq_f32_u32, float32x4_t, uint8x16_t, uint32x4_t, vreinterpretq_u32_u8};

pub fn example_reinterpret_cast(data: &[u8]) -> ([f32; 4], [u32; 4]) {
    // Assume data length is at least 16 bytes for this example
    let vector: uint8x16_t = unsafe { vld1q_u8(data.as_ptr()) }; // Load 16 u8 values

    // Reinterpret as u32
    let u32_vector: uint32x4_t = unsafe { vreinterpretq_u32_u8(vector) };
    
    // Reinterpret as f32
    let f32_vector: float32x4_t = unsafe { vreinterpretq_f32_u32(u32_vector) };

    // Create arrays to hold the results for demonstration purposes
    let mut f32_array: [f32; 4] = [0.0; 4];
    let mut u32_array: [u32; 4] = [0; 4];

    // Store the results back into arrays (if needed)
    unsafe{
        for i in 0..4 {
            f32_array[i] = *(((&f32_vector as *const float32x4_t) as *const f32).offset(i as isize)) ;
            u32_array[i] = *(((&u32_vector as *const uint32x4_t) as *const u32).offset(i as isize)) ;
        }
    }
    (f32_array, u32_array)
}


use opencv::{
    core::MatTraitConst,
    highgui::{self, WINDOW_AUTOSIZE},
    imgcodecs,
};
use std::env;

fn main() -> Result<()> {




    let data: [u8; 16] = [0, 0, 0, 1, 0, 0, 0, 0, 9, 10, 11, 12, 13, 14, 15, 16];
    
    let (f32_values, u32_values) = example_reinterpret_cast(&data);
    
    println!("F32 Values: {:?}", f32_values);
    println!("U32 Values: {:?}", u32_values);








    // let args: Vec<String> = env::args().collect();

    // if args.len() < 2 {
    //     eprintln!("Usage: {} <photo_file_path>", args[0]);
    //     return Ok(());
    // }

    // let image: opencv::prelude::Mat = imgcodecs::imread(&args[1], imgcodecs::IMREAD_COLOR)?;

    // let width = image.size()?.width;
    // let height = image.size()?.height;
    // println!("image dimensions: {}x{}", width, height);

    // highgui::named_window("hello opencv!", WINDOW_AUTOSIZE)?;
    // highgui::imshow("hello opencv!", &image)?;
    // highgui::wait_key(10000)?;


    Ok(())
}


