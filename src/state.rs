use crate::{foxdbg_channel_type_t, foxdbg_image_info_t};
use foxglove::{ChannelId, McapWriterHandle};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Mutex, OnceLock};

/// Global store mapping shannel id's to extra information needed for channels
pub static CHANNELS: Lazy<Mutex<HashMap<ChannelId, ChannelState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Global store for the Mcap handle to keep the connection alive
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

/// Manages the state of the MCAP writer handle.
///
/// This structure uses a combination of `OnceLock`, `Mutex`, and `Option` to ensure that
/// the MCAP writer is initialised only once, can be safely accessed from multiple threads,
/// and is properly closed when the application shuts down.
///
/// - `OnceLock`: Ensures that the `McapWriterHandle` is set only once. This is crucial
///   because the handle should not be replaced once it's been created.
/// - `Mutex`: Provides thread-safe mutable access to the handle. This is necessary
///   because we need to be able to `take()` the handle to close it, which requires
///   mutable access.
/// - `Option`: Allows us to `take()` the handle out of the `Mutex` to close it. When
///   `close()` is called, the `Option` is set to `None`, and the handle is dropped,
///   which in turn closes the MCAP file.
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
