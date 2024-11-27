use lib::mat_packet;
use lib::my_arm_neon;
use opencv::{
    boxed_ref::BoxedRef,
    core::{Mat, Rect, CV_8UC1, CV_8UC3},
    highgui::{self, WINDOW_AUTOSIZE},
    prelude::*,
    videoio, Result,
};
use std::thread;
use std::time::Duration;
use zmq::Context;



fn main() {
    let context = Context::new();

    // Task receiver (PULL)
    let task_receiver = context
        .socket(zmq::PULL)
        .expect("Failed to create task receiver");
    task_receiver
        .connect(&format!("tcp://{}:{}", mat_packet::HOST_IP, mat_packet::TASK_PORT))
        .expect("Failed to connect to task receiver");

    // Result sender (PUSH)
    let result_sender = context
        .socket(zmq::PUSH)
        .expect("Failed to create result sender");
    result_sender
        .connect(&format!("tcp://{}:{}", mat_packet::HOST_IP, mat_packet::RESULT_PORT))
        .expect("Failed to connect to result sender");

    println!("Compute node is ready for tasks.");

    loop {
        // Receive task

        let message = task_receiver.recv_msg(0).unwrap();
        let msg: mat_packet::MatMessage = bincode::deserialize(&message).expect("Deserialization failed");

        let frame_num = msg.number;
        let mut frame = mat_packet::message_to_mat(msg).unwrap();

        // println!("Processing Packet ID {}: {}", packet_id, packet_data);

        dbg!("frame processing begin");
        let sobel_frame = my_arm_neon::do_frame(&frame).unwrap();
        dbg!("frame complete");

        let mat_message = mat_packet::mat_to_message(&sobel_frame, frame_num, 0).unwrap();
        let serialized: Vec<u8> = bincode::serialize(&mat_message).expect("Serialization failed");

        let serialized = message; //just echo back the og frame
        result_sender
            .send(serialized, 0)
            .expect("Failed to send result");
    }
}

