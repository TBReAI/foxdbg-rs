use std::{
    ffi::{CStr, c_char},
    os::raw::{c_int, c_void},
};

use foxglove::{schemas::Log, Context, Encode};

pub fn write_channel(topic_name: &str, data: *const c_void, size: usize) {
    println!(
        "data_writer::write_channel called for topic_name: {}, size: {}",
        topic_name, size
    );

    let channel = Context::get_default()
        .get_channel_by_topic(topic_name)
        .expect("Cannel could not be found");

    let log = Log{
        message: "Hello, foxglove!".to_string(), 
        ..Default::default()
    };

    let mut buffer = vec![];
    log.encode(&mut buffer).expect("Failed encoding message");

    channel.log(&buffer);
}

pub fn write_channel_info(channel_id: c_int, data: *const c_void, size: usize) {
    println!(
        "data_writer::write_channel_info called for channel_id: {}, size: {}",
        channel_id, size
    );
    // Placeholder: In a real implementation, 'data' would be read and sent as channel info.
}
