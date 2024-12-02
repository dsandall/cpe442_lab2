use opencv::{
    core::Mat,
    highgui::{self, WINDOW_AUTOSIZE},
    prelude::*,
    videoio, Result,
};
use std::sync::{atomic::AtomicU64, Arc};
// use std::prelude::*;
use std::env;

use lib::mat_packet;

use tokio::{sync::Mutex, task::yield_now};

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use zmq::{Context, Socket};

use std::sync::atomic::Ordering;




#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <video_file_path>", args[0]);
        return Ok(());
    }
    let filename = &args[1];

    // Open the video file
    let video = videoio::VideoCapture::from_file(filename, videoio::CAP_ANY)?;
    if !video.is_opened()? {
        panic!("Error: Couldn't open video file.");
    }

    // open zeromq ports for communication with clients
    let (tx, rx, _context) = init_zmq()?;

    // screw around with Arc<Mutex<>> patterns because rust is rust
    let tx_safe = Arc::new(Mutex::new(tx));
    let tx_clone: Arc<Mutex<Socket>> = Arc::clone(&tx_safe);
    let rx_safe = Arc::new(Mutex::new(rx));
    let rx_clone: Arc<Mutex<Socket>> = Arc::clone(&rx_safe);

    let shared_counter = Arc::new(AtomicU64::new(0));
    let counter1 = Arc::clone(&shared_counter);
    let counter2 = Arc::clone(&shared_counter);

    // spawn thread for transmission
    tokio::spawn(async move { send_frames(tx_clone, video, counter1).await });

    // spawn thread for reception
    receive_frames(rx_clone, counter2).await?;



    Ok(())
}



async fn send_frames(tx_mutex: Arc<Mutex<Socket>>, mut video: videoio::VideoCapture, rx_count: Arc<AtomicU64>) -> Result<()> {
    let mut frame_count = 0;

    let tx_guard = tx_mutex.lock().await;

    loop {
        // Read the next frame
        let mut frame = Mat::default();
        if !video.read(&mut frame)? {
            println!("Video processing finished.");
            break;
        }

        // Do the actual frame stuff
        // let combined_frame = my_arm_neon::do_frame(&frame)?;
        // let combined_frame = do_networks(frame, frame_count, &tx, &rx)?;

        let mat_message = mat_packet::from_mat(&frame, frame_count, 0).unwrap();
        let serialized: Vec<u8> = bincode::serialize(&mat_message).expect("Serialization failed");
        let size = serialized.len();
        (*tx_guard)
        .send(serialized, 0)
        .expect("Failed to send task");
    
        dbg!("frame sent", frame_count );
        dbg!("with message size: ", size);
        frame_count += 1;
        

        while frame_count > rx_count.load(Ordering::SeqCst) + 8 {
            yield_now().await;
        }


    }

    Ok(())
}



async fn receive_frames(rx_mutex: Arc<Mutex<Socket>>, count: Arc<AtomicU64>) -> Result<()> {
    let start = std::time::Instant::now();
    let mut last : std::time::Instant = start;

    // Create a window to display frames
    highgui::named_window("Video Frame", WINDOW_AUTOSIZE)?;

    let mut frame_buffer: BinaryHeap<Reverse<(u64, mat_packet::MatMessage)>> = BinaryHeap::new();

    let rx_guard = rx_mutex.lock().await;


    loop {
        println!("waiting for message...");
        let bytes: zmq::Message = (*rx_guard).recv_msg(0).unwrap(); //blocking
        println!("msg recvd");
        let size = bytes.len();

        let msg: mat_packet::MatMessage =
            bincode::deserialize(&bytes).expect("Deserialization failed");
        drop(bytes);

        let rx_num = msg.number;

        dbg!("recieved # with message size:", rx_num,  size);

        // Store the message in the buffer
        if rx_num < count.load(Ordering::SeqCst) {
            // but only if you need it (if you should somehow recieve a frame you already recieved)
            break;
        } else {
            frame_buffer.push(Reverse((rx_num, msg)));
        }


        // Every 50 frames, calculate and print averages
        if rx_num % 50 == 0 {
            let now = std::time::Instant::now();
            let total_sobel_time= now.duration_since(start);
            let last_50_time = now.duration_since(last);
            
            println!(
                "Averages after {} frames: avg time to sobel: {:?}/only last 50: {:?}",
                rx_num, 
                total_sobel_time / ((rx_num.max(1)) as u32),
                last_50_time/50
            );

            last = now;

        }

        // Process messages in order
        while let Some(Reverse((number, _message))) = frame_buffer.peek() {
            if *number == count.load(Ordering::SeqCst) {
                // Pop the message from the buffer
                let Reverse((_, msg)) = frame_buffer.pop().unwrap();

                // Convert to a frame and display
                let combined_frame = Mat::try_from(&msg).unwrap();
                count.fetch_add(1,Ordering::SeqCst); // += 1
                // total_sobel_time += Duration::new(1, 0);

                highgui::imshow("Video Frame", &combined_frame).unwrap();


            } else {
                // Break if the next expected frame is not at the front of the buffer
                break;
            }
        }

        // wait minimum time before continuing loop (note: maybe make display and packet reception different threads?)
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
    tx.set_sndhwm(1)
        .expect("failed to set high water mark");

    // Result receiver (PULL)
    let rx: zmq::Socket = context
        .socket(zmq::PULL)
        .expect("Failed to create result receiver");
    rx.bind(&format!("tcp://*:{}", mat_packet::RESULT_PORT))
        .expect("Failed to bind result receiver");
    rx.set_rcvhwm(1)  // Set receive high water mark (max messages to buffer)
        .expect("Failed to set receive HWM for result receiver");

    println!("Host is ready to distribute tasks and receive results.");

    Ok((tx, rx, context))
}
