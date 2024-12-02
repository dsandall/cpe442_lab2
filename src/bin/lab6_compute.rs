use lib::mat_packet;
use lib::my_arm_neon;
use opencv::{core::Mat, prelude::*};
use zmq::Context;
fn main() {
    let context = Context::new();

    // Task receiver (PULL)
    let tx = context
        .socket(zmq::PULL)
        .expect("Failed to create task receiver");
    tx
        .connect(&format!(
            "tcp://{}:{}",
            mat_packet::HOST_IP,
            mat_packet::TASK_PORT
        ))
        .expect("Failed to connect to task receiver");
    tx.set_sndhwm(1)  // Set send high water mark (max messages to buffer)
        .expect("Failed to set receive HWM for result receiver");

    // Result sender (PUSH)
    let rx = context
        .socket(zmq::PUSH)
        .expect("Failed to create result sender");
    rx
        .connect(&format!(
            "tcp://{}:{}",
            mat_packet::HOST_IP,
            mat_packet::RESULT_PORT
        ))
        .expect("Failed to connect to result sender");
    rx.set_rcvhwm(1)  // Set receive high water mark (max messages to buffer)
        .expect("Failed to set receive HWM for result receiver");

    println!("Compute node is ready for tasks.");

    loop {
        // Receive task
        //dbg!("Message reception:");

        let message = tx.recv_msg(0).unwrap();
        let msg: mat_packet::MatMessage =
            bincode::deserialize(&message).expect("Deserialization failed");

        //dbg!(format!("{},{}", msg.data[0], msg.number));
        let frame_num = msg.number;
        let frame = Mat::try_from(&msg).unwrap();

        // println!("Processing Packet ID {}: {}", packet_id, packet_data);

        //dbg!("frame processing begin");
        // sleep(Duration::new(5,0));
        let sobel_frame = my_arm_neon::do_frame(&frame).unwrap();
        //dbg!("frame complete");

        let mat_message = mat_packet::from_mat(&sobel_frame, frame_num, 0).unwrap();

        let serialized: Vec<u8> = bincode::serialize(&mat_message).expect("Serialization failed");

        // let serialized = message; //just echo back the og frame
        rx
            .send(serialized, 0)
            .expect("Failed to send result");
    }
}
