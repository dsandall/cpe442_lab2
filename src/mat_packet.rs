use opencv::{
    core::{self, CV_8UC1, CV_8UC3},
    prelude::*,
};
use serde::{Deserialize, Serialize};

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

// traits to support comparing (and thus ordering) the packets by frame number
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

// Conversion Traits/functions
pub fn from_mat(mat: &core::Mat, number: u64, send_time: i32) -> Result<MatMessage, opencv::Error> {
    Ok(MatMessage {
        rows: mat.rows(),
        cols: mat.cols(),
        mat_type: mat.typ(),
        number,
        send_time,
        data: mat.data_bytes()?.to_vec(),
    })
}

impl TryFrom<&MatMessage> for opencv::core::Mat {
    type Error = opencv::Error;

    fn try_from(msg: &MatMessage) -> Result<Self, Self::Error> {
        // Test Assertions:
        // Validate dimensions
        if msg.rows <= 0 || msg.cols <= 0 {
            //dbg!(&msg.rows, &msg.cols, &msg.mat_type, &msg.number);
            //dbg!(&msg.data[0..8]);
            return Err(opencv::Error::new(
                opencv::core::StsOutOfRange,
                "Invalid matrix dimensions",
            ));
        }

        // Validate data size expectations
        let size = match msg.mat_type {
            CV_8UC3 => 3,
            CV_8UC1 => 1,
            _ => 0,
        };
        let expected_size = (msg.rows * msg.cols * size) as usize;
        if msg.data.len() != expected_size {
            return Err(opencv::Error::new(
                opencv::core::StsUnmatchedSizes,
                "Matrix size does not match its data buffer length",
            ));
        }
        // Test Assertions End

        unsafe {
            opencv::core::Mat::new_rows_cols_with_data_unsafe_def(
                msg.rows,
                msg.cols,
                msg.mat_type,
                msg.data.as_ptr().cast::<std::ffi::c_void>().cast_mut(),
            )
        }
    }
}
