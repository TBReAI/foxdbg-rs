use foxglove::Channel;
use foxglove::schemas::Log;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

pub fn add_channel(
    topic_name: &str,
    channel_type: c_int,
    target_hz: c_int,
) {
    let channel = Channel::<Log>::new(topic_name);

    println!("Added channel '{}' with ID {}", topic_name, channel.id());
}

pub fn add_rx_channel(topic_name: &str, channel_type: c_int) -> c_int {
    println!(
        "channel_manager::add_rx_channel called with topic: {}, type: {}",
        topic_name, channel_type
    );
    // Placeholder: return a dummy channel ID
    2
}

pub fn get_rx_channel(topic_name: *const c_char) -> c_int {
    let c_str = unsafe { CStr::from_ptr(topic_name) };
    let topic_name_str = c_str.to_str().unwrap_or("invalid_topic");
    println!(
        "channel_manager::get_rx_channel called with topic: {}",
        topic_name_str
    );
    // Placeholder: return a dummy channel ID
    2
}
