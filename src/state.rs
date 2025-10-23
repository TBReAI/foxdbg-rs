use crate::foxdbg_channel_type_t;
use foxglove::McapWriterHandle;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Mutex, OnceLock};

pub static CHANNELS: Lazy<Mutex<HashMap<String, foxdbg_channel_type_t>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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

pub static MCAP_STATE: Lazy<McapState> = Lazy::new(McapState::new);
