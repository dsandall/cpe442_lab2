use opencv::{
    boxed_ref::BoxedRef,
    core::Rect,
    highgui::{self, WINDOW_AUTOSIZE},
    prelude::*,
    videoio,
};
use opencv::{
    core::{Buffer_Access, Mat, MatTrait, MatTraitConst, CV_8UC1, CV_8UC3},
    Result,
};
use rayon::prelude::*;

use std::arch::aarch64::*;

const NUM_THREADS: usize = 4;

pub fn do_frame(frame: &Mat) -> Result<Mat> {
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

    dbg!("process frame begin earlier");

    //move these to parallel
    let mats = vec![mat1, mat2, mat3, mat4];
    let sobel_results = do_sobel_parallel(&mats)?;
    //end parallel

    dbg!("process frame begin");

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

    dbg!("process frame complete");

    Ok(combined_frame)
}

// Process Sobel in parallel
pub fn do_sobel_parallel(mats: &[BoxedRef<'_, Mat>]) -> Result<Vec<Mat>> {
    let results: Vec<Mat> = mats
        .par_iter()
        .map(|mat| to442_sobel_simd(&to442_grayscale_simd(mat).unwrap()).unwrap())
        .collect();

    // // Sequential implementation (still splits the frame)
    // let results = vec![my_arm_neon::to442_sobel_simd(&my_arm_neon::to442_grayscale_simd(&mats[0]).unwrap()).unwrap(),
    // my_arm_neon::to442_sobel_simd(&my_arm_neon::to442_grayscale_simd(&mats[1]).unwrap()).unwrap(),
    // my_arm_neon::to442_sobel_simd(&my_arm_neon::to442_grayscale_simd(&mats[2]).unwrap()).unwrap(),
    // my_arm_neon::to442_sobel_simd(&my_arm_neon::to442_grayscale_simd(&mats[3]).unwrap()).unwrap()];

    Ok(results)
}

pub fn to442_grayscale_simd(frame: &opencv::mod_prelude::BoxedRef<'_, Mat>) -> Result<Mat> {
    // Convert the frame reference to a mutable slice of `u8`
    let bgr_data: &[u8] = unsafe {
        std::slice::from_raw_parts(frame.data(), (frame.rows() * frame.cols() * 3) as usize)
    };
    assert!(
        bgr_data.len() % 12 == 0,
        "Input data length must be a multiple of 12"
    );

    // convert the output to a mutable slice
    let output: Mat =
        unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };
    let out_ptr: &mut [u8] = unsafe {
        std::slice::from_raw_parts_mut(
            output.data() as *mut u8,
            (frame.rows() * frame.cols()) as usize,
        )
    };

    // Process each chunk of 12 bytes (4 pixels * 3 channels)
    for (index, chunk) in bgr_data.chunks_exact(12).enumerate() {
        // Load the BGR bytes into separate arrays for NEON operations
        let b: [f32; 4] = [
            chunk[0].into(),
            chunk[3].into(),
            chunk[6].into(),
            chunk[9].into(),
        ]; // Blue values
        let g: [f32; 4] = [
            chunk[1].into(),
            chunk[4].into(),
            chunk[7].into(),
            chunk[10].into(),
        ]; // Green values
        let r: [f32; 4] = [
            chunk[2].into(),
            chunk[5].into(),
            chunk[8].into(),
            chunk[11].into(),
        ]; // Red values

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
            vst1q_f32(grey_vec.as_mut_ptr(), grey);

            out_ptr[index * 4] = grey_vec[0] as u8;
            out_ptr[index * 4 + 1] = grey_vec[1] as u8;
            out_ptr[index * 4 + 2] = grey_vec[2] as u8;
            out_ptr[index * 4 + 3] = grey_vec[3] as u8;
        }
    }

    Ok(output)
}

pub fn to442_sobel_simd(frame: &Mat) -> Result<Mat> {
    let input = unsafe {
        std::slice::from_raw_parts(
            frame.data() as *mut u8,
            (frame.rows() * frame.cols()) as usize,
        )
    };
    let mut output: Mat =
        unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };

    let input_2d: &[&[u8]] = &input.chunks(frame.cols() as usize).collect::<Vec<&[u8]>>();

    // Define the Sobel kernels as arrays of 8-bit signed integers
    let gx_data: [[i8; 8]; 3] = [
        [-1, 0, 1, 0, 0, 0, 0, 0], // First row of gx kernel, padded with 0s
        [-2, 0, 2, 0, 0, 0, 0, 0], // Second row of gx kernel
        [-1, 0, 1, 0, 0, 0, 0, 0], // Third row of gx kernel
    ];

    let gy_data: [[i8; 8]; 2] = [
        [1, 2, 1, 0, 0, 0, 0, 0], // First row of gy kernel
        // [0, 0, 0, 0, 0, 0, 0, 0],   //  row of gy kernel (all zeros)
        [-1, -2, -1, 0, 0, 0, 0, 0], // Second row of gy kernel
    ];

    // Load the arrays into NEON registers
    let gx: (int8x8_t, int8x8_t, int8x8_t) = unsafe {
        (
            vld1_s8(gx_data[0].as_ptr()), // Load first row into int8x8_t
            vld1_s8(gx_data[1].as_ptr()), // Load second row into int8x8_t
            vld1_s8(gx_data[2].as_ptr()), // Load third row into int8x8_t
        )
    };

    let gy: (int8x8_t, int8x8_t) = unsafe {
        (
            vld1_s8(gy_data[0].as_ptr()), // Load first row into int8x8_t
            // vld1_s8(gy_data[].as_ptr()), // Load second row into int8x8_t
            vld1_s8(gy_data[1].as_ptr()), // Load third row into int8x8_t
        )
    };

    let mut out_x = 0;

    let input_2d = &input_2d[1..input_2d.len() - 1]; // don't sobel the first or last rows
    for (out_y, row) in input_2d.iter().enumerate() {
        let row = &row[1..row.len() - 1]; // don't sobel the first or last columns
                                          // for value in row.chunks(6) {
        for chunk in row.chunks(6).enumerate() {
            unsafe {
                // load next u8 (x8)
                let surround: [uint8x8_t; 3] = [
                    vld1_u8((&(chunk.1)[0] as *const u8).offset((-frame.cols() as isize) - 1)), // row above
                    vld1_u8((&(chunk.1)[0] as *const u8).offset(-1)), // row
                    vld1_u8((&(chunk.1)[0] as *const u8).offset(frame.cols() as isize - 1)), // row below
                ];

                // u8 to signed 16 bit greyscale pixels, 3x8 grid (3 vectors of 8)
                let signed_surround = surround.map(|x| vreinterpretq_s16_u16(vmovl_u8(x)));

                #[cfg(feature = "debug")]
                println!(
                    "\n\nSigned surrounding pixels: {:?}, {:?}, {:?}",
                    signed_surround[0], signed_surround[1], signed_surround[2]
                );

                let mut x_kernel = { [vmovl_s8(gx.0), vmovl_s8(gx.1), vmovl_s8(gx.2)] };
                let mut y_kernel = { [vmovl_s8(gy.0), vmovl_s8(gy.1)] };

                for i in 0..chunk.1.len() {
                    #[cfg(feature = "debug")]
                    println!(
                        "x kern: {:?}, {:?}, {:?}",
                        x_kernel[0], x_kernel[1], x_kernel[2]
                    );
                    #[cfg(feature = "debug")]
                    println!("y kern: {:?}, {:?}", y_kernel[0], y_kernel[1]);

                    // perform x kernel convolution for first position
                    let mut acc: int16x8_t = vdupq_n_s16(0); // Initialize all 8 elements to 0
                    acc = vmlaq_s16(acc, signed_surround[0], x_kernel[0]);
                    #[cfg(feature = "debug")]
                    println!("x1 acc {:?}", acc);

                    acc = vmlaq_s16(acc, signed_surround[1], x_kernel[1]);
                    #[cfg(feature = "debug")]
                    println!("x2 acc {:?}", acc);

                    acc = vmlaq_s16(acc, signed_surround[2], x_kernel[2]);
                    #[cfg(feature = "debug")]
                    println!("x3 acc {:?}", acc);

                    let x_kernel_sum: i16 = vaddvq_s16(acc); // This sums all the elements in the vector and returns a scalar value
                    #[cfg(feature = "debug")]
                    println!("X kernel sum: {}", x_kernel_sum);

                    // perform y kernel convolution for first position
                    acc = vdupq_n_s16(0); // Initialize all 8 elements to 0
                    acc = vmlaq_s16(acc, signed_surround[0], y_kernel[0]);
                    acc = vmlaq_s16(acc, signed_surround[2], y_kernel[1]); // note the indexes are slightly different due to the blank row in kernel y
                    let y_kernel_sum = vaddvq_s16(acc);
                    #[cfg(feature = "debug")]
                    println!("Y kernel sum: {}", y_kernel_sum);

                    // save the results into the output frame
                    let magnitude = (x_kernel_sum.abs() + y_kernel_sum.abs()).min(255) as u8;
                    *(output.at_2d_mut::<u8>(out_y as i32 + 1, out_x + 1 + i as i32)?) = magnitude;
                    #[cfg(feature = "debug")]
                    println!(
                        "Stored magnitude ({}) at (x: {}, y: {})",
                        magnitude, x_kernel_sum, y_kernel_sum
                    );

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

    Ok(output)
}
