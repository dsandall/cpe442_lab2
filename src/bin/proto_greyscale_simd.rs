use std::arch::aarch64::{
    float32x4_t, vaddq_f32, vld1q_f32, vmulq_n_f32, vst1q_f32
};

use opencv::{core::MatTraitConst};

use opencv::{
    boxed_ref::BoxedRef, core::{Mat, Rect, CV_8UC1}, highgui::{self, WINDOW_AUTOSIZE}, prelude::*, videoio, Result
};

fn to442_grayscale_SIMD(frame: &opencv::mod_prelude::BoxedRef<'_, Mat>) -> Result<Mat> {

    // Convert the frame reference to a mutable slice of `u8`
    let bgr_data: &[u8] = unsafe { std::slice::from_raw_parts(frame.data(), (frame.rows() * frame.cols() * 3) as usize) };

    // convert the output to a mutable slice
    let output: Mat = unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };
    let out_ptr: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(output.data() as *mut u8, (frame.rows() * frame.cols()) as usize) };


    // process_bgr_to_f32_vectors(&bgr_data, &mut out_ptr)?;

    assert!(bgr_data.len() % 12 == 0, "Input data length must be a multiple of 12");

    // Process each chunk of 12 bytes (4 pixels * 3 channels)
    for (index, chunk) in bgr_data.chunks_exact(12).enumerate() {
        // Load the BGR bytes into separate arrays for NEON operations
        let b: [f32; 4] = [chunk[0].into(), chunk[3].into(), chunk[6].into(), chunk[9].into()]; // Blue values
        let g: [f32; 4] = [chunk[1].into(), chunk[4].into(), chunk[7].into(), chunk[10].into()]; // Green values
        let r: [f32; 4] = [chunk[2].into(), chunk[5].into(), chunk[8].into(), chunk[11].into()]; // Red values

        unsafe {
            let mut b: float32x4_t = vld1q_f32(b.as_ptr()); 
            let mut g: float32x4_t = vld1q_f32(g.as_ptr()); 
            let mut r: float32x4_t = vld1q_f32(r.as_ptr()); 
            
            b = vmulq_n_f32(b, 0.0722);
            g = vmulq_n_f32(g, 0.7152);
            r = vmulq_n_f32(r, 0.2126);

            let grey: float32x4_t = vaddq_f32(r, vaddq_f32(b, g)); // 4 pixels of grey scale 


            let mut grey_vec: Vec<f32> = vec![];
            vst1q_f32(&mut grey_vec as *mut _ as *mut f32, grey);

            out_ptr[index * 4 + 0] = grey_vec[0] as u8;

        }
        
    }

    Ok(output)
}


fn process_bgr_to_f32_vectors(bgr_data: &[u8], grey_mat: &mut [u8]) -> Result<()> {

    assert!(bgr_data.len() % 12 == 0, "Input data length must be a multiple of 12");

    // Process each chunk of 12 bytes (4 pixels * 3 channels)
    for (index, chunk) in bgr_data.chunks_exact(12).enumerate() {
        // Load the BGR bytes into separate arrays for NEON operations
        let b: [f32; 4] = [chunk[0].into(), chunk[3].into(), chunk[6].into(), chunk[9].into()]; // Blue values
        let g: [f32; 4] = [chunk[1].into(), chunk[4].into(), chunk[7].into(), chunk[10].into()]; // Green values
        let r: [f32; 4] = [chunk[2].into(), chunk[5].into(), chunk[8].into(), chunk[11].into()]; // Red values

        unsafe {
            let mut b: float32x4_t = vld1q_f32(b.as_ptr()); 
            let mut g: float32x4_t = vld1q_f32(g.as_ptr()); 
            let mut r: float32x4_t = vld1q_f32(r.as_ptr()); 
            
            b = vmulq_n_f32(b, 0.0722);
            g = vmulq_n_f32(g, 0.7152);
            r = vmulq_n_f32(r, 0.2126);

            let grey: float32x4_t = vaddq_f32(r, vaddq_f32(b, g)); // 4 pixels of grey scale 


            let mut grey_vec: Vec<f32> = vec![];
            vst1q_f32(&mut grey_vec as *mut _ as *mut f32, grey);

            grey_mat[index * 4 + 0] = grey_vec[0] as u8;

        }
        
    }

    Ok(())
}


use opencv::imgcodecs;
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

    let grey = to442_grayscale(BoxedRef);

    highgui::named_window("hello opencv!", WINDOW_AUTOSIZE)?;
    highgui::imshow("hello opencv!", &image)?;
    highgui::wait_key(10000)?;

    Ok(())
}