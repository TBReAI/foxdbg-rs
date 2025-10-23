use std::ffi::c_void;

use foxglove::bytes::{BufMut, Bytes};
use foxglove::schemas::{CompressedImage, Timestamp};
use foxglove::{Context, Encode, PartialMetadata};
use turbojpeg::{Compressor, Image, PixelFormat};

use crate::channel_schemas::{Bool, Float, Integer};

use crate::state::{self, ChannelInfo};
use crate::{foxdbg_channel_type_t, foxdbg_image_info_t};

pub fn write_channel(topic_name: &str, data: *const c_void, size: usize) {
    let channels = state::CHANNELS.lock().unwrap();
    let channel_info = channels.get(topic_name).expect("Channel info not found");

    let channel = Context::get_default()
        .get_channel_by_topic(topic_name)
        .expect("Channel could not be found");

    let mut buf = vec![];

    match channel_info.channel_type {
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_FLOAT => write_float(&mut buf, data),
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_INTEGER => write_int(&mut buf, data),
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_BOOLEAN => write_bool(&mut buf, data),
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_IMAGE => {
            write_image(&mut buf, data, size, &channel_info.channel_info)
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POINTCLOUD => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_CUBES => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LINES => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POSE => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_TRANSFORM => {}
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LOCATION => {}
    }

    channel.log(&buf)
}

fn write_image(
    buf: &mut impl BufMut,
    data: *const c_void,
    data_size: usize,
    channel_info: &ChannelInfo,
) {
    let image_info = match channel_info {
        ChannelInfo::CompressedImageInfo(image_info) => image_info,
        ChannelInfo::NoInfo() => {
            panic!("Attempted to write image to channel without setting channel info")
        }
    };

    let pixel_format = match image_info.channels {
        1 => PixelFormat::GRAY,
        3 => PixelFormat::RGB,
        4 => PixelFormat::RGBA,
        _ => {
            eprintln!("Unsupported channel count: {}", image_info.channels);
            return;
        }
    };

    // interpret the raw pointer as a slice of bytes
    let raw_slice = unsafe { std::slice::from_raw_parts(data as *const u8, data_size) };

    let pitch = image_info.width as usize * image_info.channels as usize;
    let image = Image {
        pixels: raw_slice,
        width: image_info.width as usize,
        pitch: pitch,
        height: image_info.height as usize,
        format: pixel_format,
    };

    // Compress with TurboJPEG
    let mut compressor = Compressor::new().expect("Failed to create compressor");
    compressor.set_quality(25).unwrap();
    compressor.set_subsamp(turbojpeg::Subsamp::Sub2x2).unwrap();

    let jpeg_data = compressor.compress_to_vec(image).unwrap();

    CompressedImage {
        timestamp: None,
        frame_id: "".to_string(),
        data: Bytes::copy_from_slice(&jpeg_data),
        format: "JPEG".to_string(),
    }
    .encode(buf)
    .unwrap();
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

pub fn write_channel_info(topic_name: &str, data: *const c_void, size: usize) {
    let mut channels = state::CHANNELS.lock().unwrap();
    let channel_state = channels
        .get_mut(topic_name)
        .expect("Channel info not found");

    let channel_info = match channel_state.channel_type {
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_IMAGE => {
            ChannelInfo::CompressedImageInfo(unsafe { *(data as *const foxdbg_image_info_t) })
        }
        _ => ChannelInfo::NoInfo(),
    };

    channel_state.channel_info = channel_info;
}
