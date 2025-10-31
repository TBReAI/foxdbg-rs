use crate::state;
use log;
use std::time::{SystemTime, UNIX_EPOCH};

/// Initialises the foxdbg-rs system.
///
/// This function should be called once at the beginning of the application's lifecycle.
/// It performs two main tasks:
///
/// 1.  Starts the Foxglove WebSocket server in a separate thread, which allows clients
///     to connect and receive data.
/// 2.  Creates a new MCAP file with a timestamped name and initialises the global
///     `MCAP_STATE` with the writer handle. This allows data to be written to the
///     MCAP file from anywhere in the application.
pub fn init() {
    log::info!("foxdbg-rs initialized");

    // Start the Foxglove WebSocket server in a background thread. The server will
    // continue to run until the application exits.
    if let Err(e) = foxglove::WebSocketServer::new().start_blocking() {
        log::error!("Failed to start WebSocket server: {}", e);
    }

    // Create a new MCAP file for recording log data. The file is named with the
    // current UNIX timestamp to ensure uniqueness.
    let unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time is set before UNIX_EPOCH")
        .as_secs();

    let mcap_file = format!("{:?}.mcap", unix_time);
    match foxglove::McapWriter::new().create_new_buffered_file(&mcap_file) {
        Ok(handle) => {
            log::info!("MCAP writer started writing to '{}'", mcap_file);
            // Initialise the global MCAP_STATE with the writer handle. This prevents the
            // handle from being dropped, which would close the MCAP file.
            state::MCAP_STATE.init(handle);
        }
        Err(e) => {
            log::error!(
                "Failed to start MCAP Writer for mcap file '{}': {}",
                mcap_file,
                e
            );
        }
    };
}

pub fn shutdown() {
    log::info!("Foxdbg-rs shutting down");
    state::MCAP_STATE.close();
}
