use bincode;
use opencv::{core, prelude::*};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use opencv::boxed_ref::BoxedRef;

pub const TASK_PORT: &str = "5555"; // For sending tasks
pub const RESULT_PORT: &str = "5556"; // For receiving results
pub const HOST_IP: &str = "10.0.1.152"; // Replace with host's IP

use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug)] // Include Debug for better debug output
pub struct MatMessage {
    pub rows: i32,
    pub cols: i32,
    pub mat_type: i32,  // e.g., CV_8UC3
    pub number: u64,    // the frame number
    pub send_time: i32, // should be time/Instant type though
    pub data: Vec<u8>,
}

impl Eq for MatMessage {}

impl PartialEq for MatMessage {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl Ord for MatMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.number.cmp(&other.number)
    }
}

impl PartialOrd for MatMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}



pub fn mat_to_message(mat: &core::Mat, number: u64, send_time: i32) -> Result<MatMessage, opencv::Error> {
    let rows = mat.rows();
    let cols = mat.cols();
    let mat_type = mat.typ();
    let data = mat.data_bytes()?.to_vec(); // Extract raw data as Vec<u8>

    let mat_message = MatMessage {
        rows,
        cols,
        mat_type,
        number,
        send_time,
        data,
    };

    // dbg!(&mat_message.data[0..8]);

    Ok(mat_message)
}

pub fn message_to_mat(msg: &MatMessage) -> Result<core::Mat, opencv::Error> {
    dbg!(&msg.rows, &msg.cols, &msg.mat_type, &msg.number);
    dbg!(&msg.data[0..8]);

    // let mat = unsafe {opencv::core::Mat::new_rows_cols_with_bytes::<T>(
    //     msg.rows,
    //     msg.cols,
    //     &msg.data
    // )}

    unsafe {
        opencv::core::Mat::new_rows_cols_with_data_unsafe_def(
            msg.rows,
            msg.cols,
            msg.mat_type,
            msg.data.as_ptr().cast::<std::ffi::c_void>().cast_mut(),
        )
    }
}

