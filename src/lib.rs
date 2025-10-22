#![allow(non_camel_case_types)]
use std::ffi::{CStr, c_void};
use std::os::raw::c_char;
use std::slice;

// Declare internal modules
mod channel_manager;
mod core;
mod data_writer;

// STRUCTS

// Make sure memory layout matches C
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct foxdbg_color_t {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct foxdbg_vector3_t {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct foxdbg_vector4_t {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct foxdbg_pose_t {
    pub position: foxdbg_vector3_t,
    pub orientation: foxdbg_vector3_t,
    pub color: foxdbg_color_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct foxdbg_cube_t {
    pub position: foxdbg_vector3_t,
    pub size: foxdbg_vector3_t,
    pub orientation: foxdbg_vector3_t,
    pub color: foxdbg_color_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct foxdbg_transform_t {
    pub id: *const c_char,
    pub parent_id: *const c_char,
    pub position: foxdbg_vector3_t,
    pub orientation: foxdbg_vector3_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct foxdbg_line_t {
    pub start: foxdbg_vector3_t,
    pub end: foxdbg_vector3_t,
    pub color: foxdbg_color_t,
    pub thickness: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct foxdbg_location_t {
    pub timestamp_sec: u32,
    pub timestamp_nsec: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct foxdbg_image_info_t {
    pub width: i32,
    pub height: i32,
    pub channels: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum foxdbg_channel_type_t {
    FOXDBG_CHANNEL_TYPE_IMAGE,
    FOXDBG_CHANNEL_TYPE_POINTCLOUD,
    FOXDBG_CHANNEL_TYPE_CUBES,
    FOXDBG_CHANNEL_TYPE_LINES,
    FOXDBG_CHANNEL_TYPE_POSE,
    FOXDBG_CHANNEL_TYPE_TRANSFORM,
    FOXDBG_CHANNEL_TYPE_LOCATION,
    FOXDBG_CHANNEL_TYPE_FLOAT,
    FOXDBG_CHANNEL_TYPE_INTEGER,
    FOXDBG_CHANNEL_TYPE_BOOLEAN,
}

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
    channel_type: foxdbg_channel_type_t,
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
    channel_type: foxdbg_channel_type_t,
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
    topic_name: *const std::os::raw::c_char,
    data: *const std::os::raw::c_void,
    size: usize,
) {
    println!("Called Write channel info!");
    let topic_name_str = unsafe {
        CStr::from_ptr(topic_name)
            .to_str()
            .unwrap_or("invalid_topic")
    };
    data_writer::write_channel_info(topic_name_str, data, size);
}

