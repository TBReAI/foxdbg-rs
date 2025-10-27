#![allow(non_camel_case_types)]
use std::ffi::{CStr, c_int};

// Declare internal modules
mod channels;
mod core;
mod state;

// C-exported types
pub mod types;
use foxglove::ChannelId;
use state::CHANNELS;
pub use types::*;

// FFI functions

/// Initialises the debugging system and starts the server thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn foxdbg_init() {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::init_from_env(env);
    core::init();
}

/// Shuts down the server and cleans up resources.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn foxdbg_shutdown() {
    core::shutdown();
}

/// Creates a new channel (topic) to publish data to Foxglove. It returns a channel ID for
/// use with other functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn foxdbg_add_channel(
    topic_name: *const std::os::raw::c_char,
    channel_type: foxdbg_channel_type_t,
    _target_hz: std::os::raw::c_int,
) -> std::os::raw::c_int {
    let topic_name_str = unsafe {
        CStr::from_ptr(topic_name)
            .to_str()
            .unwrap_or("invalid_topic")
    };
    channels::manager::add_channel(topic_name_str, channel_type) as i32
}

/// Writes a data payload to a specified channel.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn foxdbg_write_channel(
    channel_id: c_int,
    data: *const std::os::raw::c_void,
    size: usize,
) {
    let channels = CHANNELS.lock().unwrap();
    let channel_state = channels.get(&ChannelId::new(channel_id as u64)).unwrap();
    unsafe { channels::writer::write_channel(channel_state, data, size) };
}

/// Writes metadata for a channel, used for types like images to specify dimensions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn foxdbg_write_channel_info(
    channel_id: c_int,
    data: *const std::os::raw::c_void,
    size: usize,
) {
    let mut channels = CHANNELS.lock().unwrap();
    let mut channel_state = channels
        .get_mut(&ChannelId::new(channel_id as u64))
        .unwrap();
    unsafe { channels::writer::write_channel_info(&mut channel_state, data, size) };
}
