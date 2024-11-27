use zmq::Context;
use std::thread;
use std::time::Duration;
use opencv::{
    boxed_ref::BoxedRef, core::{Mat, Rect, CV_8UC1, CV_8UC3}, highgui::{self, WINDOW_AUTOSIZE}, prelude::*, videoio, Result
};
use my_arm_neon::MatMessage;


const HOST_IP: &str = "10.0.1.152"; // Replace with host's IP
const TASK_RECEIVER_PORT: &str = "5555";
const RESULT_SENDER_PORT: &str = "5556";

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
        let msg: MatMessage = bincode::deserialize(&message).expect("Deserialization failed");
        let frame_num = msg.number;
        let mut frame = my_arm_neon::message_to_mat(msg);

        // println!("Processing Packet ID {}: {}", packet_id, packet_data);

        dbg!("frame processing begin");
        let sobel_frame = my_arm_neon::do_frame(&frame).unwrap();
        dbg!("frame complete");

        let mat_message = my_arm_neon::mat_to_message(&sobel_frame, frame_num, 0);
        let serialized: Vec<u8> = bincode::serialize(&mat_message).expect("Serialization failed");
        
        result_sender.send(serialized, 0).expect("Failed to send result");
    }
}
