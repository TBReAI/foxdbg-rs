pub mod scene;
pub mod sensor;
pub mod telemetry;

mod helpers;

use foxglove::Context;
use helpers::data_as_ref;
use std::cell::RefCell;
use std::ffi::c_void;

use crate::{foxdbg_channel_type_t, foxdbg_image_info_t};
use crate::state::{ChannelInfo, ChannelState};

thread_local! {
    /// A thread-local buffer used for serialising log data before sending it to Foxglove.
    ///
    /// This buffer is reused for each `write_channel` call on the same thread to avoid
    /// the performance overhead of allocating a new buffer every time. The buffer is
    /// cleared before each use and has an initial capacity of 10MB to reduce the
    /// likelihood of reallocations for large messages. Once the buffer has been reallocated, 
    /// it will not shrink
    static LOG_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(10 * 1024 * 1024)); // 10MB initial capacity
}

/// Writes a data payload to a specified channel.
///
/// This function takes a `ChannelState` and a raw C pointer to the data, serialises
/// the data into a thread-local buffer based on the channel type, and then logs the
/// data to the appropriate Foxglove channel.
///
/// # Safety
///
/// This function is `unsafe` because it delegates to other `unsafe` functions that
/// dereference the raw `data` pointer. The caller must ensure that the `data` pointer
/// is valid, non-null, and that `size` correctly corresponds to the size of the data.
///
/// # Arguments
///
/// * `channel_state` - A reference to the `ChannelState` for the channel to write to.
/// * `data` - A raw C pointer to the data payload.
/// * `size` - The size of the data payload in bytes.
pub unsafe fn write_channel(channel_state: &ChannelState, data: *const c_void, size: usize) {
    let channel = Context::get_default()
        .get_channel_by_topic(&channel_state.channel_topic)
        .expect("Channel could not be found");

    LOG_BUFFER.with(|buf_cell| {
        let mut buf = buf_cell.borrow_mut();
        buf.clear();

        unsafe {
            match channel_state.channel_type {
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_FLOAT => {
                    telemetry::write_float(&mut *buf, data, size)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_INTEGER => {
                    telemetry::write_int(&mut *buf, data, size)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_BOOLEAN => {
                    telemetry::write_bool(&mut *buf, data, size)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_IMAGE => {
                    sensor::write_image(&mut *buf, data, size, &channel_state.channel_info)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POINTCLOUD => {
                    sensor::write_pointcloud(&mut *buf, data, size)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_CUBES => {
                    scene::write_cubes(&mut *buf, data, size, &channel_state.channel_topic)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LINES => {
                    scene::write_lines(&mut *buf, data, size, &channel_state.channel_topic)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POSE => {
                    scene::write_pose(&mut *buf, data, size, &channel_state.channel_topic)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_TRANSFORM => {
                    scene::write_transform(&mut *buf, data, size)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LOCATION => {
                    sensor::write_location(&mut *buf, data, size)
                }
            }
        }

        channel.log(&buf);
    });
}

/// Writes metadata for a channel, such as image dimensions.
///
/// This function is used to provide additional information about a channel that is
/// required for proper decoding or visualisation. For example, for an image channel,
/// this function should be called with a `foxdbg_image_info_t` struct to provide the
/// image dimensions.
///
/// # Safety
///
/// This function is `unsafe` because it calls `data_as_ref`, which dereferences a raw
/// pointer. The caller must ensure that the `data` pointer is valid, non-null, and
/// that `size` correctly corresponds to the size of the metadata struct.
///
/// # Arguments
///
/// * `channel_state` - A mutable reference to the `ChannelState` to update.
/// * `data` - A raw C pointer to the metadata struct.
/// * `size` - The size of the metadata struct in bytes.
pub unsafe fn write_channel_info(channel_state: &mut ChannelState, data: *const c_void, size: usize) {

    let channel_info = if let Some(image_info) =
        unsafe { data_as_ref::<foxdbg_image_info_t>(data, size) }
    {
        match channel_state.channel_type {
            crate::foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_IMAGE => {
                ChannelInfo::CompressedImageInfo(*image_info)
            }
            _ => ChannelInfo::NoInfo(),
        }
    } else {
        ChannelInfo::NoInfo()
    };

    channel_state.channel_info = channel_info;
}
