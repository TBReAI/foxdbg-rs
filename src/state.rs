use crate::{foxdbg_channel_type_t, foxdbg_image_info_t};
use foxglove::{ChannelId, McapWriterHandle};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Mutex, OnceLock};

pub static CHANNELS: Lazy<Mutex<HashMap<ChannelId, ChannelState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub static MCAP_STATE: Lazy<McapState> = Lazy::new(McapState::new);

#[derive(Debug)]
pub struct ChannelState {
    pub channel_type: foxdbg_channel_type_t,
    pub channel_info: ChannelInfo,
    pub channel_topic: String,
}

#[derive(Debug)]
pub enum ChannelInfo {
    CompressedImageInfo(foxdbg_image_info_t),
    NoInfo(),
}

// This is disgusting
// OnceLock to stop the handle dropping
// Mutex to allow mutable borrows
// Option so ownership can be transfered out the mutex
pub struct McapState {
    writer: OnceLock<Mutex<Option<McapWriterHandle<BufWriter<File>>>>>,
}

impl McapState {
    fn new() -> Self {
        McapState {
            writer: OnceLock::new(),
        }
    }

    pub fn init(&self, handle: McapWriterHandle<BufWriter<File>>) {
        self.writer
            .set(Mutex::new(Some(handle)))
            .expect("MCAP writer has already been initialized");
    }

    pub fn close(&self) {
        if let Some(lock) = self.writer.get() {
            if let Some(writer) = lock.lock().expect("Failed to lock writer mutex").take() {
                writer.close().expect("Failed to close MCAP writer");
            }
        }
    }
}
