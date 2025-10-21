use std::ffi::CStr;

// Declare internal modules
mod core;
mod channel_manager;
mod data_writer;

// FFI functions

/// Initializes the debugging system and starts the server thread.
#[unsafe(no_mangle)]
pub extern "C" fn foxdbg_init() {
    core::init();
}

/// Polls for received data callbacks. This should be called periodically in your application's main loop to process incoming messages.
#[unsafe(no_mangle)]
pub extern "C" fn foxdbg_update() {
    core::update();
}

/// Shuts down the server and cleans up resources.
#[unsafe(no_mangle)]
pub extern "C" fn foxdbg_shutdown() {
    core::shutdown();
}

/// Creates a new channel (topic) to publish data to Foxglove. It returns a channel ID for
/// use with other functions.
#[unsafe(no_mangle)]
pub extern "C" fn foxdbg_add_channel(
    topic_name: *const std::os::raw::c_char,
    channel_type: std::os::raw::c_int,
    target_hz: std::os::raw::c_int,
) -> std::os::raw::c_int {
    let topic_name_str = unsafe {
        CStr::from_ptr(topic_name)
            .to_str()
            .unwrap_or("invalid_topic")
    };
    channel_manager::add_channel(topic_name_str, channel_type, target_hz);
    1
}

/// Creates a channel to receive data from Foxglove (for simple types like float, integer, boolean).
#[unsafe(no_mangle)]
pub extern "C" fn foxdbg_add_rx_channel(
    topic_name: *const std::os::raw::c_char,
    channel_type: std::os::raw::c_int,
) -> std::os::raw::c_int {
    
    let topic_name_str = unsafe {
        CStr::from_ptr(topic_name)
            .to_str()
            .unwrap_or("invalid_topic")
    };

    channel_manager::add_rx_channel(topic_name_str, channel_type)
}

/// Writes a data payload to a specified channel.
#[unsafe(no_mangle)]
pub extern "C" fn foxdbg_write_channel(
    topic_name: *const std::os::raw::c_char,
    data: *const std::os::raw::c_void,
    size: usize,
) {

    let topic_name_str = unsafe {
        CStr::from_ptr(topic_name)
            .to_str()
            .unwrap_or("invalid_topic")
    };

    data_writer::write_channel(topic_name_str, data, size);
}

/// Writes metadata for a channel, used for types like images to specify dimensions.
#[unsafe(no_mangle)]
pub extern "C" fn foxdbg_write_channel_info(
    channel_id: std::os::raw::c_int,
    data: *const std::os::raw::c_void,
    size: usize,
) {
    data_writer::write_channel_info(channel_id, data, size);
}
