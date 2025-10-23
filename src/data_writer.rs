use std::{ffi::c_void, sync::Arc};

use foxglove::{
    Context, Encode, RawChannel,
};

use crate::channel_schemas::{Bool, Float, Integer};

use crate::foxdbg_channel_type_t;
use crate::state;

pub fn write_channel(topic_name: &str, data: *const c_void, size: usize) {
    let channels = state::CHANNELS.lock().unwrap();
    let channel_type = channels.get(topic_name).expect("Channel info not found");

    let channel = Context::get_default()
        .get_channel_by_topic(topic_name)
        .expect("Channel could not be found");

    match channel_type {
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_FLOAT => {
            write_float(channel, unsafe { *(data as *const f32) })
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_INTEGER => {
            write_int(channel, unsafe { *(data as *const u32) })
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_BOOLEAN => {
            write_bool(channel, unsafe { *(data as *const bool) })
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_IMAGE => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POINTCLOUD => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_CUBES => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LINES => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POSE => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_TRANSFORM => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LOCATION => {}
    }
}

fn write_int(channel: Arc<RawChannel>, data: u32) {
    println!("{}", data);
    let mut buf: Vec<u8> = vec![];
    Integer { value: data }.encode(&mut buf);

    channel.log(&buf)
}

fn write_bool(channel: Arc<RawChannel>, data: bool) {
    let mut buf: Vec<u8> = vec![];
    Bool { value: data }.encode(&mut buf);

    channel.log(&buf)
}

fn write_float(channel: Arc<RawChannel>, data: f32) {
    let mut buf: Vec<u8> = vec![];
    Float { value: data }.encode(&mut buf);

    channel.log(&buf)
}

pub fn write_channel_info(topic_name: &str, data: *const c_void, size: usize) {}
