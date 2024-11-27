use std::sync::Arc;
use std::time::Instant;
use std::{env, time::Duration};
use opencv::{
    boxed_ref::BoxedRef,
    core::{Mat, Rect, CV_8UC1},
    highgui::{self, WINDOW_AUTOSIZE},
    prelude::*,
    videoio, Result,
};

use lib::mat_packet;
use lib::my_arm_neon;

use tokio::sync::Mutex;

use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task;
use zmq::{Context, Socket};

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

    // screw around with Arc<Mutex<>> patterns because rust is rust
    let tx_safe = Arc::new(Mutex::new(tx));
    let tx_clone: Arc<Mutex<Socket>> = Arc::clone(&tx_safe);
    let rx_safe = Arc::new(Mutex::new(rx));
    let rx_clone: Arc<Mutex<Socket>> = Arc::clone(&rx_safe);

    // spawn thread for transmission
    tokio::spawn(async move { send_frames(tx_clone, video).await });

    // spawn thread for reception
    receive_frames(rx_clone).await;

    Ok(())
}

async fn send_frames(tx_mutex: Arc<Mutex<Socket>>, mut video: videoio::VideoCapture) -> Result<()> {
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

        let mat_message = mat_packet::mat_to_message(&frame, frame_count, 0).unwrap();
        let serialized: Vec<u8> = bincode::serialize(&mat_message).expect("Serialization failed");
        (*tx_guard)
            .send(serialized, 0)
            .expect("Failed to send task");

        frame_count += 1;
    }

    Ok(())
}


use std::collections::BinaryHeap;
use std::cmp::Reverse;

async fn receive_frames(rx_mutex: Arc<Mutex<Socket>>) -> Result<()> {
    let mut frame_count = 0;
    let mut total_sobel_time = std::time::Duration::new(0, 0);

    // Create a window to display frames
    highgui::named_window("Video Frame", WINDOW_AUTOSIZE)?;

    let mut next_frame_number = 0;
    let mut frame_buffer: BinaryHeap<Reverse<(u64, mat_packet::MatMessage)>> = BinaryHeap::new();

    let rx_guard = rx_mutex.lock().await;

    loop {
        println!("waiting for message...");
        let bytes: zmq::Message = (*rx_guard).recv_msg(0).unwrap(); //blocking
        println!("msg recvd");

        let msg: mat_packet::MatMessage = bincode::deserialize(&bytes).expect("Deserialization failed");
        drop(bytes);

        dbg!(msg.number);

        // Store the message in the buffer
        frame_buffer.push(Reverse((msg.number, msg)));

        // Process messages in order
        while let Some(Reverse((number, message))) = frame_buffer.peek() {
            if *number == next_frame_number {
                // Pop the message from the buffer
                let Reverse((_, msg)) = frame_buffer.pop().unwrap();

                // Convert to a frame and display
                let combined_frame = mat_packet::message_to_mat(&msg).unwrap();
                frame_count += 1;
                total_sobel_time += Duration::new(1, 0);

                highgui::imshow("Video Frame", &combined_frame).unwrap();

                next_frame_number += 1;

                // Every 50 frames, calculate and print averages
                if frame_count % 50 == 0 {
                    let avg_sobel_time = total_sobel_time / frame_count;
                    println!(
                        "Averages after {} frames: Sobel: {:?}",
                        frame_count, avg_sobel_time
                    );
                }
            } else {
                // Break if the next expected frame is not at the front of the buffer
                break;
            }
        }

        // Wait for 30ms between frames
        if highgui::wait_key(1).unwrap() == 27 {
            // Exit if the 'ESC' key is pressed
            println!("ESC key pressed. Exiting...");
            break;
        }
    }

    Ok(())
}


fn init_zmq() -> Result<(Socket, Socket, Context)> {
    let context = Context::new();

    // Task sender (PUSH)
    let tx: zmq::Socket = context
        .socket(zmq::PUSH)
        .expect("Failed to create task sender");
    tx.bind(&format!("tcp://*:{}", mat_packet::TASK_PORT))
        .expect("Failed to bind task sender");

    // Result receiver (PULL)
    let rx: zmq::Socket = context
        .socket(zmq::PULL)
        .expect("Failed to create result receiver");
    rx.bind(&format!("tcp://*:{}", mat_packet::RESULT_PORT))
        .expect("Failed to bind result receiver");

    println!("Host is ready to distribute tasks and receive results.");

    Ok((tx, rx, context))
}
