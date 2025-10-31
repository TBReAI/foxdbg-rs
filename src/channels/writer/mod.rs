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
    static LOG_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(1024 * 1024)); // 1MB initial capacity
}

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
