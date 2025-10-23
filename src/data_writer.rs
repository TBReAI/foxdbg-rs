use std::{ffi::c_void, sync::Arc};

use foxglove::bytes::BufMut;
use foxglove::{Context, Encode, RawChannel};

use crate::channel_schemas::{Bool, Float, Integer};

use crate::foxdbg_channel_type_t;
use crate::state;

pub fn write_channel(topic_name: &str, data: *const c_void, size: usize) {
    let channels = state::CHANNELS.lock().unwrap();
    let channel_type = channels.get(topic_name).expect("Channel info not found");

    let channel = Context::get_default()
        .get_channel_by_topic(topic_name)
        .expect("Channel could not be found");

    let mut buf = vec![];

    match channel_type {
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_FLOAT => write_float(&mut buf, data),
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_INTEGER => write_int(&mut buf, data),
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_BOOLEAN => write_bool(&mut buf, data),
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_IMAGE => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POINTCLOUD => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_CUBES => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LINES => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POSE => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_TRANSFORM => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LOCATION => {}
    }

    channel.log(&buf)
}

fn write_int(buf: &mut impl BufMut, data: *const c_void) {
    let value = unsafe { *(data as *const u32) };
    Integer { value }.encode(buf).unwrap();
}

fn write_bool(buf: &mut impl BufMut, data: *const c_void) {
    let value = unsafe { *(data as *const bool) };
    Bool { value }.encode(buf).unwrap();
}

fn write_float(buf: &mut impl BufMut, data: *const c_void) {
    let value = unsafe { *(data as *const f32) };
    Float { value }.encode(buf).unwrap();
}

pub fn write_channel_info(topic_name: &str, data: *const c_void, size: usize) {}
