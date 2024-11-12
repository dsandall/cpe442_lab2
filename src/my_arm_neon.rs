// #![cfg(target_arch = "aarch64")]

// use std::env;
// use rayon::prelude::*;
use opencv::{
    core::{Buffer_Access, Mat, MatTrait, MatTraitConst, CV_8UC1}, 
    // highgui::{self, WINDOW_AUTOSIZE}, prelude::*, videoio, 
    Result
};
// use std::time::Instant;


use std::arch::aarch64::*;

pub fn to442_grayscale_simd(frame: &opencv::mod_prelude::BoxedRef<'_, Mat>) -> Result<Mat> {

    // Convert the frame reference to a mutable slice of `u8`
    let bgr_data: &[u8] = unsafe { std::slice::from_raw_parts(frame.data(), (frame.rows() * frame.cols() * 3) as usize) };
    assert!(bgr_data.len() % 12 == 0, "Input data length must be a multiple of 12");

    // convert the output to a mutable slice
    let output: Mat = unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };
    let out_ptr: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(output.data() as *mut u8, (frame.rows() * frame.cols()) as usize) };


    // Process each chunk of 12 bytes (4 pixels * 3 channels)
    for (index, chunk) in bgr_data.chunks_exact(12).enumerate() {
        // Load the BGR bytes into separate arrays for NEON operations
        let b: [f32; 4] = [chunk[0].into(), chunk[3].into(), chunk[6].into(), chunk[9].into()]; // Blue values
        let g: [f32; 4] = [chunk[1].into(), chunk[4].into(), chunk[7].into(), chunk[10].into()]; // Green values
        let r: [f32; 4] = [chunk[2].into(), chunk[5].into(), chunk[8].into(), chunk[11].into()]; // Red values

        unsafe {
            // 4 pixels split into 3 vectors
            let mut b: float32x4_t = vld1q_f32(b.as_ptr()); 
            let mut g: float32x4_t = vld1q_f32(g.as_ptr()); 
            let mut r: float32x4_t = vld1q_f32(r.as_ptr()); 
            
            // multiplication by scalar coefficients
            b = vmulq_n_f32(b, 0.0722);
            g = vmulq_n_f32(g, 0.7152);
            r = vmulq_n_f32(r, 0.2126);
            
            
            // add em back up into one 4 pixel vector
            let grey: float32x4_t = vaddq_f32(r, vaddq_f32(b, g)); 

            let mut grey_vec: [f32; 4] = [0.0; 4];
            vst1q_f32( grey_vec.as_mut_ptr(), grey);

            out_ptr[index * 4] = grey_vec[0] as u8;
            out_ptr[index * 4 + 1] = grey_vec[1] as u8;
            out_ptr[index * 4 + 2] = grey_vec[2] as u8;
            out_ptr[index * 4 + 3] = grey_vec[3] as u8;
        }
        
    }

    Ok(output)
}



pub fn to442_sobel_simd(frame: &Mat) -> Result<Mat> {

    // let a = frame.input_output_array();

    let input = unsafe { std::slice::from_raw_parts(frame.data() as *mut u8, (frame.rows() * frame.cols()) as usize) };
    let mut output: Mat = unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };

    let input_2d: &[&[u8]] = &input.chunks(frame.cols() as usize).collect::<Vec<&[u8]>>();

    // Define the Sobel kernels as arrays of 8-bit signed integers
    let gx_data: [[i8; 8]; 3] = [
        [-1, 0, 1, 0, 0, 0, 0, 0], // First row of gx kernel, padded with 0s
        [-2, 0, 2, 0, 0, 0, 0, 0], // Second row of gx kernel
        [-1, 0, 1, 0, 0, 0, 0, 0], // Third row of gx kernel
    ];

    let gy_data: [[i8; 8]; 2] = [
        [1, 2, 1, 0, 0, 0, 0, 0],   // First row of gy kernel
        // [0, 0, 0, 0, 0, 0, 0, 0],   //  row of gy kernel (all zeros)
        [-1, -2, -1, 0, 0, 0, 0, 0],// Second row of gy kernel
    ];

    // Load the arrays into NEON registers
    let gx: (int8x8_t, int8x8_t, int8x8_t) = unsafe {(
        vld1_s8(gx_data[0].as_ptr()), // Load first row into int8x8_t
        vld1_s8(gx_data[1].as_ptr()), // Load second row into int8x8_t
        vld1_s8(gx_data[2].as_ptr()) // Load third row into int8x8_t
    )};

    let gy: (int8x8_t, int8x8_t) = unsafe {(
        vld1_s8(gy_data[0].as_ptr()), // Load first row into int8x8_t
        // vld1_s8(gy_data[].as_ptr()), // Load second row into int8x8_t
        vld1_s8(gy_data[1].as_ptr()) // Load third row into int8x8_t
    )};
        


    let mut out_x = 0;

    let input_2d = &input_2d[1..input_2d.len() - 1]; // don't sobel the first or last rows
    for (out_y, row) in input_2d.iter().enumerate() {

        let row = &row[1..row.len() - 1]; // don't sobel the first or last columns
        // for value in row.chunks(6) {
        for chunk in row.chunks(6).enumerate() {

            unsafe {
                // load next u8 (x8) 
                let surround:[uint8x8_t; 3] = [
                    vld1_u8((&(chunk.1)[0] as *const u8).offset((- frame.cols() as isize) -1)), // row above
                    vld1_u8((&(chunk.1)[0] as *const u8).offset(-1)), // row 
                    vld1_u8((&(chunk.1)[0] as *const u8).offset(frame.cols() as isize -1)), // row below
                ];      
                
                // u8 to signed 16 bit greyscale pixels, 3x8 grid (3 vectors of 8)
                let signed_surround = surround.map(|x| vreinterpretq_s16_u16(vmovl_u8(x)));    

                #[cfg(feature = "debug")] println!("\n\nSigned surrounding pixels: {:?}, {:?}, {:?}", signed_surround[0], signed_surround[1], signed_surround[2]);

                let mut x_kernel = {[
                    vmovl_s8(gx.0),
                    vmovl_s8(gx.1),
                    vmovl_s8(gx.2)
                ]};
                let mut y_kernel = {[
                    vmovl_s8(gy.0),
                    vmovl_s8(gy.1)
                ]};                                     

                for i in 0..chunk.1.len() {

                    #[cfg(feature = "debug")] println!("x kern: {:?}, {:?}, {:?}",x_kernel[0], x_kernel[1], x_kernel[2]);
                    #[cfg(feature = "debug")] println!("y kern: {:?}, {:?}", y_kernel[0], y_kernel[1]);

                    // perform x kernel convolution for first position
                    let mut acc: int16x8_t = vdupq_n_s16(0); // Initialize all 8 elements to 0
                    acc = vmlaq_s16(acc, signed_surround[0], x_kernel[0]);
                    #[cfg(feature = "debug")] println!("x1 acc {:?}",acc);

                    acc = vmlaq_s16(acc, signed_surround[1], x_kernel[1]);
                    #[cfg(feature = "debug")] println!("x2 acc {:?}",acc);

                    acc = vmlaq_s16(acc, signed_surround[2], x_kernel[2]);
                    #[cfg(feature = "debug")] println!("x3 acc {:?}",acc);

                    let x_kernel_sum: i16 = vaddvq_s16(acc); // This sums all the elements in the vector and returns a scalar value
                    #[cfg(feature = "debug")] println!("X kernel sum: {}", x_kernel_sum);

                    // perform y kernel convolution for first position
                    acc = vdupq_n_s16(0); // Initialize all 8 elements to 0
                    acc = vmlaq_s16(acc, signed_surround[0], y_kernel[0]);
                    acc = vmlaq_s16(acc, signed_surround[2], y_kernel[1]); // note the indexes are slightly different due to the blank row in kernel y
                    let y_kernel_sum = vaddvq_s16(acc);
                    #[cfg(feature = "debug")] println!("Y kernel sum: {}", y_kernel_sum);


                    // save the results into the output frame
                    let magnitude = (x_kernel_sum.abs() + y_kernel_sum.abs()).min(255) as u8;
                    *(output.at_2d_mut::<u8>(out_y as i32 +1, out_x +1 +i as i32)?) = magnitude;
                    #[cfg(feature = "debug")] println!("Stored magnitude ({}) at (x: {}, y: {})", magnitude, x_kernel_sum, y_kernel_sum);


                    // shift kernels over by one pixel (vector rotate elements)
                    let r_shift_kernel_row = |kernel_row| vextq_s16::<7>(kernel_row, kernel_row);
                    x_kernel = x_kernel.map(r_shift_kernel_row);
                    y_kernel = y_kernel.map(r_shift_kernel_row);
                }   
            }
            out_x += chunk.1.len() as i32;
        }
        out_x = 0;
    }



    // //for each inner pixel
    // for y in 1..(frame.rows() - 1) {
    //     for x in 1..(frame.cols() - 1) {
    //         // let pixel = (output.at_2d_mut::<u8>(y, x)?, x, y); 
    
    //         unsafe {
    //             // load next u8 (x8) 
    //             let surround: [uint8x8_t; 3] = [
    //                 vld1_u8(input.as_ptr().offset((y as isize - 1) * frame.cols() as isize + (x - 1) as isize)), // row above
    //                 vld1_u8(input.as_ptr().offset((y as isize) * frame.cols() as isize + (x - 1) as isize)), // row center
    //                 vld1_u8(input.as_ptr().offset((y as isize + 1) * frame.cols() as isize + (x - 1) as isize)), // row below
    //             ];
    
    //             // Debug: Print the raw surrounding pixel values
    //             println!(
    //                 "Surrounding pixels (raw): {:?}, {:?}, {:?}",
    //                 surround[0], surround[1], surround[2]
    //             );
    
    //             // u8 to signed 16 bit greyscale pixels, 3x8 grid (3 vectors of 8)
    //             let signed_surround = surround.map(|x| vreinterpretq_s16_u16(vmovl_u8(x)));
    
    //             // Debug: Print the signed surrounding pixel values
    //             println!(
    //                 "Signed surrounding pixels: {:?}, {:?}, {:?}",
    //                 signed_surround[0], signed_surround[1], signed_surround[2]
    //             );
    //             println!(
    //                 "x kern: {:?}, {:?}, {:?}",
    //                 x_kernel[0], x_kernel[1], x_kernel[2]
    //             );
    //             println!(
    //                 "y kern: {:?}, {:?}",
    //                 y_kernel[0], y_kernel[1]
    //             );
    
    //             // perform x kernel convolution for first position
    //             let mut acc: int16x8_t = vdupq_n_s16(0); // Initialize all 8 elements to 0
    //             acc = vmlaq_s16(acc, signed_surround[0], x_kernel[0]);
    //             println!(
    //                 "x1 acc {:?}",
    //                 acc
    //             );
    //             acc = vmlaq_s16(acc, signed_surround[1], x_kernel[1]);
    //             println!(
    //                 "x2 acc {:?}",
    //                 acc
    //             );
    //             acc = vmlaq_s16(acc, signed_surround[2], x_kernel[2]);
    //             println!(
    //                 "x3 acc {:?}",
    //                 acc
    //             );
    //             let x_kernel_sum = vaddvq_s16(acc); // This sums all the elements in the vector and returns a scalar value
    
    //             // Debug: Print the result of the x kernel sum
    //             println!("X kernel sum: {}", x_kernel_sum);
    
    //             // perform y kernel convolution for first position
    //             acc = vdupq_n_s16(0); // Initialize all 8 elements to 0
    //             acc = vmlaq_s16(acc, signed_surround[0], y_kernel[0]);
    //             acc = vmlaq_s16(acc, signed_surround[2], y_kernel[1]); // note the indexes are slightly different due to the blank row in kernel y
    //             let y_kernel_sum = vaddvq_s16(acc);
    
    //             // Debug: Print the result of the y kernel sum
    //             println!("Y kernel sum: {}", y_kernel_sum);
    
    //             // save the results into the output frame
    //             let magnitude = (x_kernel_sum.abs() + y_kernel_sum.abs()).min(255) as u8;
    
    //             // Debug: Print the magnitude before clamping to 255
    //             println!("Magnitude (before clamping to 255): {}", magnitude);
    
    //             *(output.at_2d_mut::<u8>(y, x)?) = magnitude;
    
    //             // Debug: Confirm that the magnitude has been stored
    //             println!("Stored magnitude at (x: {}, y: {})", x, y);
    //         }

    //         println!(
    //             "end of pixel ({},{})\n\n", x, y
    //         );
    //     }
    //     println!(
    //         "end of row ({})\n\n", y
    //     );
    // }
    




    // let gy_data_old: [[i8; 8]; 3] = [
    //     [1, 2, 1, 0, 0, 0, 0, 0],   // First row of gy kernel
    //     [0, 0, 0, 0, 0, 0, 0, 0],   //  row of gy kernel (all zeros)
    //     [-1, -2, -1, 0, 0, 0, 0, 0],// Second row of gy kernel
    // ];

    // // compute sobel for each inner pixel
    // for y in 1..(frame.rows() - 1) {
    //     for x in 1..(frame.cols() - 1) {

    //     // Initialize accumulators for the x and y gradients
    //     let (sum_x, sum_y) = 
    //         (0..3).flat_map(|ky| { // Iterate over the y kernel indices
    //             (0..3).map(move |kx| { // Iterate over the x kernel indices
    //                 // get the greyscale value for that kernel pixel
    //                 let pixel: u8 = *frame.at_2d::<u8>(y + ky - 1, x + kx - 1).unwrap();
                    
    //                 // Calculate contributions to the x and y gradients using the Sobel kernels
    //                 let gradient_x = pixel as i16 * gx_data[ky as usize][kx as usize] as i16; // Contribution for Gx
    //                 let gradient_y = pixel as i16 * gy_data_old[ky as usize][kx as usize] as i16; // Contribution for Gy
                    
    //                 // Return the contributions as a tuple
    //                 (gradient_x, gradient_y)
    //             })
    //         })
    //         // Accumulate the gradient contributions into sum_x and sum_y
    //         .fold((0i16, 0i16), |(acc_x, acc_y), (dx, dy)| {
    //             (acc_x + dx, acc_y + dy)
    //         });


    //         let magnitude = (sum_x.abs() + sum_y.abs()).min(255) as u8;

    //         *(output.at_2d_mut::<u8>(y, x)?) = magnitude;

    //     }
    // }



    Ok(output)
}