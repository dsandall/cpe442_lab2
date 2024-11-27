use std::{env, time::Duration};
use opencv::{
    boxed_ref::BoxedRef, core::{Mat, Rect, CV_8UC1}, highgui::{self, WINDOW_AUTOSIZE}, prelude::*, videoio, Result
};
use std::time::Instant;
use my_arm_neon::MatMessage;
use tokio::sync::Mutex;
use std::sync::Arc;


#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <video_file_path>", args[0]);
        return Ok(());
    }
    let filename = &args[1];

    // Open the video file
    let mut video = videoio::VideoCapture::from_file(filename, videoio::CAP_ANY)?;
    if !video.is_opened()? {
        panic!("Error: Couldn't open video file.");
    }



    // open zeromq ports for communication with clients
    let (tx, rx, context) = init_zmq()?;

    let tx_safe = Arc::new(Mutex::new(tx));
    let tx_clone: Arc<Mutex<Socket>> = Arc::clone(&tx_safe);

    let rx_safe = Arc::new(Mutex::new(rx));
    let rx_clone: Arc<Mutex<Socket>> = Arc::clone(&rx_safe);


    // let tx_future = send_frames(&tx, video);
    // let rx_future = receive_frames(&rx);
    
    tokio::spawn(async move {
        send_frames(tx_clone, video).await
    });
    
    receive_frames(rx_clone).await;
    

    // rx_future.await;
    // tx_future.await;
    
    
    

    // loop {
    //     // Read the next frame
    //     let mut frame = Mat::default();
    //     if !video.read(&mut frame)? {
    //         println!("Video processing finished.");
    //         break;
    //     } else if frame.empty() {
    //         println!("Empty frame detected. Video might have ended.");
    //         break;
    //     }

    //     // Do the actual frame stuff
    //     // let combined_frame = my_arm_neon::do_frame(&frame)?;
    //     // let combined_frame = do_networks(frame, frame_count, &tx, &rx)?;

    //     let mat_message = my_arm_neon::mat_to_message(&frame, frame_count, 0);
    //     let serialized: Vec<u8> = bincode::serialize(&mat_message).expect("Serialization failed");
    //     tx.send(serialized, 0).expect("Failed to send task");

    //     println!("waiting for message...");
    //     let bytes : zmq::Message;
    //     let bytes = rx.recv_msg(0).unwrap(); //blocking?
    //     println!("msg recvd");

    //     let msg: MatMessage = bincode::deserialize(&bytes).expect("Deserialization failed");
    //     let combined_frame = my_arm_neon::message_to_mat(msg);
    //     frame_count += 1;

    //     // // Display the frames in the windows
    //     highgui::imshow("Video Frame", &combined_frame)?;
    //     // highgui::imshow("Video Frame2", &frame)?;

    //     // Wait for 30ms between frames
    //     if highgui::wait_key(1)? == 27 {
    //         // Exit if the 'ESC' key is pressed
    //         println!("ESC key pressed. Exiting...");
    //         break;
    //     }

    //     // Every 50 frames, calculate and print averages
    //     if frame_count % 50 == 0 {
    //         let avg_sobel_time = total_sobel_time / frame_count;
    //         println!(
    //             "Averages after {} frames: Sobel: {:?}",
    //             frame_count, avg_sobel_time
    //         );
    //     }
    // }
    
    Ok(())
}

async fn send_frames(tx_mutex: Arc<Mutex<Socket>>, mut video : videoio::VideoCapture) -> Result<()> {
    let mut frame_count = 0;

    let tx_guard = tx_mutex.lock().await;

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


        // Do the actual frame stuff
        // let combined_frame = my_arm_neon::do_frame(&frame)?;
        // let combined_frame = do_networks(frame, frame_count, &tx, &rx)?;

        let mat_message = my_arm_neon::mat_to_message(&frame, frame_count, 0);
        let serialized: Vec<u8> = bincode::serialize(&mat_message).expect("Serialization failed");
        (*tx_guard).send(serialized, 0).expect("Failed to send task");


        frame_count += 1;
    }

    Ok(())


}
async fn receive_frames(rx_mutex: Arc<Mutex<Socket>>) -> Result<()> {
    let mut frame_count = 0;
    let mut total_sobel_time = std::time::Duration::new(0, 0);
    
    // Create a window to display frames
    highgui::named_window("Video Frame", WINDOW_AUTOSIZE)?;
    // highgui::named_window("Video Frame2", WINDOW_AUTOSIZE)?;

    let rx_guard = rx_mutex.lock().await;


    loop {
    
        println!("waiting for message...");
        let bytes: zmq::Message = (*rx_guard).recv_msg(0).unwrap(); //blocking
        println!("msg recvd");

        let msg: MatMessage = bincode::deserialize(&bytes).expect("Deserialization failed");
        drop(bytes);
        let combined_frame = my_arm_neon::message_to_mat(msg);

        frame_count += 1;
        total_sobel_time += Duration::new(1, 0);   

        // // Display the frames in the windows
        highgui::imshow("Video Frame", &combined_frame).unwrap();
        // highgui::imshow("Video Frame2", &frame)?;

        // Wait for 30ms between frames
        if highgui::wait_key(1).unwrap() == 27 {
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


// fn do_networks<'a>(mut frame: Mat, number:u32, tx: &Socket, rx: &Socket) -> Result<Mat>{
    
//     let (rows, cols) = (frame.rows(), frame.cols());
//     dbg!(&frame);
    
//     tx.send(my_arm_neon::mat_to_message(&frame, number), 0).expect("Failed to send task");

//     println!("waiting for message...");
//     let bytes : zmq::Message;
//     let bytes = rx.recv_msg(0).unwrap(); //blocking?
//     println!("msg recvd");

//     let msg: MatMessage = bincode::deserialize(&bytes).expect("Deserialization failed");
//     let mut frame = my_arm_neon::message_to_mat(msg);
//     Ok(frame)
// }

// {

// }

