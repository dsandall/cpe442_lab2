use lib::mat_packet;
use lib::my_arm_neon;
use opencv::{core::Mat, prelude::*};
use zmq::Context;

fn main() {
    let context = Context::new();

    // Task receiver (PULL)
    let task_receiver = context
        .socket(zmq::PULL)
        .expect("Failed to create task receiver");
    task_receiver
        .connect(&format!(
            "tcp://{}:{}",
            mat_packet::HOST_IP,
            mat_packet::TASK_PORT
        ))
        .expect("Failed to connect to task receiver");

    // Result sender (PUSH)
    let result_sender = context
        .socket(zmq::PUSH)
        .expect("Failed to create result sender");
    result_sender
        .connect(&format!(
            "tcp://{}:{}",
            mat_packet::HOST_IP,
            mat_packet::RESULT_PORT
        ))
        .expect("Failed to connect to result sender");

    println!("Compute node is ready for tasks.");

    loop {
        // Receive task
        dbg!("Message reception:");

        let message = task_receiver.recv_msg(0).unwrap();
        let msg: mat_packet::MatMessage =
            bincode::deserialize(&message).expect("Deserialization failed");

        dbg!("next are same?");
        dbg!(msg.data[0]);
        let frame_num = msg.number;
        let frame = Mat::try_from(&msg).unwrap();
        dbg!(frame.data_bytes().unwrap()[0]);

        // println!("Processing Packet ID {}: {}", packet_id, packet_data);

        dbg!("frame processing begin");
        let sobel_frame = my_arm_neon::do_frame(&frame).unwrap();
        dbg!("frame complete");

        let mat_message = mat_packet::from_mat(&sobel_frame, frame_num, 0).unwrap();

        let serialized: Vec<u8> = bincode::serialize(&mat_message).expect("Serialization failed");

        // let serialized = message; //just echo back the og frame
        result_sender
            .send(serialized, 0)
            .expect("Failed to send result");
    }
}
