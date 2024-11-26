use zmq::Context;
use std::thread;
use std::time::Duration;
use opencv::{
    boxed_ref::BoxedRef, core::{Mat, Rect, CV_8UC1, CV_8UC3}, highgui::{self, WINDOW_AUTOSIZE}, prelude::*, videoio, Result
};

const HOST_IP: &str = "10.0.1.152"; // Replace with host's IP
const TASK_RECEIVER_PORT: &str = "5555";
const RESULT_SENDER_PORT: &str = "5556";

fn to_zmq_message(m : Mat) -> zmq::Message {
    let mat_slice: &[u8] = unsafe {
        std::slice::from_raw_parts(
            m.data(),
            (m.rows() * m.cols() * 3) as usize,
        )
    };
    zmq::Message::from(mat_slice)
}
fn main() {
    let context = Context::new();

    // Task receiver (PULL)
    let task_receiver = context.socket(zmq::PULL).expect("Failed to create task receiver");
    task_receiver.connect(&format!("tcp://{}:{}", HOST_IP, TASK_RECEIVER_PORT))
        .expect("Failed to connect to task receiver");

    // Result sender (PUSH)
    let result_sender = context.socket(zmq::PUSH).expect("Failed to create result sender");
    result_sender.connect(&format!("tcp://{}:{}", HOST_IP, RESULT_SENDER_PORT))
        .expect("Failed to connect to result sender");

    println!("Compute node is ready for tasks.");

    loop {
        // Receive task

        let message =  task_receiver.recv_msg(0).unwrap();
        let mut frame = my_arm_neon::message_to_mat(message.to_vec());


        
        // let mut frame = unsafe {Mat::new_rows_cols_with_data(320, 180, &message).unwrap()};

        // match_length(&[rows, cols], &message.len(), 1)?;


        // let task = task_receiver.recv_string(0).expect("Failed to receive task").unwrap();
        // let mut parts = task.splitn(2, '|');
        // let packet_id = parts.next().unwrap();
        // let packet_data = parts.next().unwrap();

        // println!("Processing Packet ID {}: {}", packet_id, packet_data);

        // Simulate processing
        // thread::sleep(Duration::from_secs(1));
        // let processed_data = format!("{} processed", packet_data);

        // Send result back to host
        // let result = format!("{}|{}", packet_id, processed_data);

        dbg!("frame processing begin");
        let sobel_frame = my_arm_neon::do_frame(&frame).unwrap();
        dbg!("frame complete");

        result_sender.send( my_arm_neon::mat_to_message(&sobel_frame), 0).expect("Failed to send result");
    }
}
