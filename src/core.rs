use foxglove::{self, McapWriterHandle};
use std::{
    fs::File,
    io::BufWriter,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

// This is disgusting
// OnceLock to stop the handle dropping
// Mutex to allow mutable borrows
// Option so ownership can be transfered out the mutex
static MCAP_WRITER: OnceLock<Mutex<Option<McapWriterHandle<BufWriter<File>>>>> = OnceLock::new();

pub fn init() {
    println!("FoxDbg initialized!");

    // Starts a WebSocketServer on a separate tokio thread
    foxglove::WebSocketServer::new()
        .start_blocking()
        .expect("Server failed to start");

    // Creates a mcap buffer writer
    let unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time is set before UNIX_EPOCH")
        .as_secs();
    let handle = foxglove::McapWriter::new()
        .create_new_buffered_file(format!("{:?}.mcap", unix_time))
        .expect("Failed to create writer");

    // The mcap writer handle will be unregistered when dropped, so want to stop that
    MCAP_WRITER
        .set(Mutex::new(Some(handle)))
        .expect("MCAP_WRITER already initialized");
}

pub fn update() {
    println!("core::update() called");
}

pub fn shutdown() {
    println!("core::shutdown() called");
    if let Some(lock) = MCAP_WRITER.get() {
        lock.lock()
            .expect("Failed to lock writer mutex")
            .take()
            .expect("Failed to take writer out of mutex")
            .close()
            .expect("Failed to close writer");
    };
}
