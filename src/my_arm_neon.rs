// #![cfg(target_arch = "aarch64")]

// use std::arch::aarch64::{vld1q_u8, vaddq_u32, vmlaq_f32, vdupq_n_f32, vst1_u8, vld1q_f32, vadd_s8};
// use opencv::{
//     boxed_ref::BoxedRef, core::{Mat, CV_8UC1}, prelude::*, Result
// };

// pub fn to442_grayscale(frame: &opencv::mod_prelude::BoxedRef<'_, Mat>) -> Result<Mat> {
//     let mut output: Mat = unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };

//     // Convert the frame reference to a mutable slice of `u8`
//     let data_slice: &[u8] = unsafe { std::slice::from_raw_parts(frame.data(), (frame.rows() * frame.cols() * 3) as usize) };
//     let output_slice: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(output.data_mut(), (output.rows() * output.cols()) as usize) };

//     // Process 16 pixels (48 bytes) at a time using NEON
//     let pixel_count = frame.cols() * frame.rows();
//     let chunks = pixel_count / 16; // Number of full chunks
//     let remainder = pixel_count % 16; // Remainder pixels

//     // Use SIMD for the chunks of 16 pixels
//     for chunk in 0..chunks {
//         let start_index = chunk * 16 * 3; // Each pixel has 3 channels (BGR)
//         unsafe{
        
//             // Load 16 pixels (48 bytes)
//             let b_vec = vld1q_u8(data_slice[start_index..(start_index + 16 * 3)].as_ptr()); // B channel
//             let g_vec = vld1q_u8(data_slice[start_index + 1..start_index + 16 * 3].as_ptr()); // G channel
//             let r_vec = vld1q_u8(data_slice[start_index + 2..start_index + 16 * 3].as_ptr()); // R channel

//             // Convert to float and apply the grayscale formula
//             let b_float = unsafe{ vdupq_n_f32(0.2126) };
//             let g_float = unsafe{ vdupq_n_f32(0.7152) };
//             let r_float = unsafe{ vdupq_n_f32(0.0722) };

//             let gray_float = vmlaq_f32(
//                 vmlaq_f32(vdupq_n_f32(0.0), vld1q_f32(b_vec.as_ptr() as *const f32), b_float),
//                 vld1q_f32(g_vec.as_ptr() as *const f32),
//                 g_float
//             );

//             let gray_float = vmlaq_f32(gray_float, vld1q_f32(r_vec.as_ptr() as *const f32), r_float);

//             // Store the result back into the output slice
//             let gray_u8: [u8; 16] = std::mem::transmute(gray_float);
//             vst1_u8(output_slice[start_index / 3..].as_mut_ptr().add(chunk * 16), gray_u8.as_ptr());
//         }
//     }

//     // Handle the remainder pixels
//     for i in (chunks * 16 * 3)..(pixel_count * 3) {
//         let b: f32 = data_slice[i as usize].into(); // Blue channel
//         let g = data_slice[(i + 1) as usize].into(); // Green channel
//         let r: f32 = data_slice[(i + 2) as usize].into(); // Red channel

//         // Apply the grayscale formula
//         let gray_value = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u8;

//         // Set the pixel value in the output matrix
//         output_slice[i / 3] = gray_value;
//     }

//     Ok(output)
// }


use std::arch::aarch64::{vld1q_u8, vreinterpretq_f32_u32, float32x4_t, uint8x16_t, uint32x4_t, vreinterpretq_u32_u8};

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

fn main() {
    let data: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    
    let (f32_values, u32_values) = example_reinterpret_cast(&data);
    
    println!("F32 Values: {:?}", f32_values);
    println!("U32 Values: {:?}", u32_values);
}
