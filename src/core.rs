use crate::state;
use std::time::{SystemTime, UNIX_EPOCH};

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
    state::MCAP_STATE.init(handle);
}

pub fn update() {
    println!("core::update() called");
}

pub fn shutdown() {
    println!("core::shutdown() called");
    state::MCAP_STATE.close();
}
