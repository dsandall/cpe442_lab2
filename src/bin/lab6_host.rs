use std::env;
use opencv::{
    boxed_ref::BoxedRef, core::{Mat, Rect, CV_8UC1}, highgui::{self, WINDOW_AUTOSIZE}, prelude::*, videoio, Result
};
use std::time::Instant;

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

    let (tx, rx, context) = init_zmq()?;

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
        // let combined_frame = my_arm_neon::do_frame(&frame)?;

        let combined_frame = do_networks(frame, &tx, &rx)?;


        // Handle timing tracking
        let sobel_duration = start_sobel.elapsed();
        total_sobel_time += sobel_duration;
        frame_count += 1;

        // // Display the frames in the windows
        highgui::imshow("Video Frame", &combined_frame)?;
        // highgui::imshow("Video Frame2", &frame)?;

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

use zmq::{Context, Socket};
use tokio::task;
use tokio::sync::mpsc;
use std::collections::HashMap;

const TASK_SENDER_PORT: &str = "5555"; // For sending tasks
const RESULT_RECEIVER_PORT: &str = "5556"; // For receiving results


fn init_zmq() -> Result<(Socket, Socket, Context)>{
    let context = Context::new();

    // Task sender (PUSH)
    let tx: zmq::Socket = context.socket(zmq::PUSH).expect("Failed to create task sender");
    tx.bind(&format!("tcp://*:{}", TASK_SENDER_PORT)).expect("Failed to bind task sender");

    // Result receiver (PULL)
    let rx: zmq::Socket = context.socket(zmq::PULL).expect("Failed to create result receiver");
    rx.bind(&format!("tcp://*:{}", RESULT_RECEIVER_PORT)).expect("Failed to bind result receiver");

    println!("Host is ready to distribute tasks and receive results.");

    Ok((tx, rx, context))
}

/*
pub trait ToZmqMessage {
    fn to_zmq_message(&self) -> zmq::Message;
}
*/
fn to_zmq_message(m : Mat) -> zmq::Message {
    let mat_slice: &[u8] = unsafe {
        std::slice::from_raw_parts(
            m.data(),
            (m.rows() * m.cols() * 3) as usize,
        )
    };
    zmq::Message::from(mat_slice)
}

/*
fn from_zmq_message <'a> (msg : zmq::Message , rows: i32 , cols: i32,) -> BoxedRef<'a, Mat>  {
    // let output: Mat = unsafe { opencv::core::Mat::from_slice_2d(frame.rows(), frame.cols(), CV_8UC1)? };
    // let output: Mat = unsafe { opencv::core::Mat::new_rows_cols(frame.rows(), frame.cols(), CV_8UC1)? };
    // let out_ptr: &mut [u8] = unsafe { std::slic       e::from_raw_parts_mut(output.data() as *mut u8, (frame.rows() * frame.cols()) as usize) };

    // Mat::new_rows_cols_with_data(rows, cols, &(msg.to_vec()) ).expect("message conversion to mat failed")
}
*/

fn do_networks<'a>(mut frame: Mat, tx: &Socket, rx: &Socket) -> Result<Mat>{
    
    let (rows, cols) = (frame.rows(), frame.cols());
    dbg!(&frame);
    
    
    tx.send(my_arm_neon::mat_to_message(&frame), 0).expect("Failed to send task");
    // tx.send(to_zmq_message(frame), 0).expect("Failed to send task");

    let msg : zmq::Message;
    /*  
    loop {            
        if let Ok(()) = rx.recv(0, &msg).ok() {
            // let mut parts = result.splitn(2, '|');
            // let packet_id: usize = parts.next().unwrap().parse().unwrap();
            // let processed_data = parts.next().unwrap().to_string();

            // result_tx_clone.send((packet_id, processed_data)).await.unwrap();
            // println!("Received result for Packet ID {}", packet_id);
            println!("msg recvd");
            Ok(msg.from_zmq_message())        
        }
    }
    */
    println!("waiting for message...");
    let msg = rx.recv_msg(0).unwrap(); //blocking?
    // let mut parts = result.splitn(2, '|');
    // let packet_id: usize = parts.next().unwrap().parse().unwrap();
    // let processed_data = parts.next().unwrap().to_string();

    // result_tx_clone.send((packet_id, processed_data)).await.unwrap();
    // println!("Received result for Packet ID {}", packet_id);
    println!("msg recvd");
    

    // let output: Mat = unsafe { opencv::core::Mat::new_rows_cols(rows, cols, CV_8UC1)? };
    // let out_ptr: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(output.data() as *mut u8, (frame.rows() * frame.cols()) as usize) };
    // *out_ptr = msg;    

    // let mut frame = unsafe {Mat::new_rows_cols_with_data(640, 480, &msg).unwrap()};
    // Ok(frame.clone_pointee())        
    
    let mut frame = my_arm_neon::message_to_mat(msg.to_vec());
    Ok(frame)




}


