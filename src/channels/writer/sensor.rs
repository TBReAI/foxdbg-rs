use crate::state::ChannelInfo;
use crate::types::{foxdbg_location_t, foxdbg_vector4_t};
use foxglove::Encode;
use foxglove::bytes::{BufMut, Bytes};
use foxglove::schemas::{CompressedImage, LocationFix, PackedElementField, PointCloud, Timestamp};
use std::ffi::c_void;
use std::mem;
use std::slice;
use turbojpeg::{Compressor, Image, PixelFormat};

use super::helpers::data_as_ref;

// --- Writers ---

pub(super) unsafe fn write_location(buf: &mut impl BufMut, data: *const c_void, data_size: usize) {
    if let Some(location_data) = unsafe { data_as_ref::<foxdbg_location_t>(data, data_size) } {
        let timestamp = Timestamp::new(location_data.timestamp_sec, location_data.timestamp_nsec);

        LocationFix {
            timestamp: Some(timestamp),
            frame_id: "world".to_owned(),
            latitude: location_data.latitude,
            longitude: location_data.longitude,
            altitude: location_data.altitude,
            position_covariance: vec![0.0; 9],
            position_covariance_type: 0,
            color: None,
        }
        .encode(buf)
        .unwrap();
    }
}

pub(super) unsafe fn write_pointcloud(
    buf: &mut impl BufMut,
    data: *const c_void,
    data_size: usize,
) {
    let raw_bytes = unsafe { slice::from_raw_parts(data as *const u8, data_size) };

    PointCloud {
        timestamp: None,
        frame_id: "world".to_owned(),
        pose: None,
        point_stride: mem::size_of::<foxdbg_vector4_t>() as u32,
        fields: vec![
            PackedElementField {
                name: "x".to_owned(),
                offset: 0,
                r#type: 7,
            },
            PackedElementField {
                name: "y".to_owned(),
                offset: 4,
                r#type: 7,
            },
            PackedElementField {
                name: "z".to_owned(),
                offset: 8,
                r#type: 7,
            },
            PackedElementField {
                name: "intensity".to_owned(),
                offset: 12,
                r#type: 7,
            },
        ],
        data: Bytes::copy_from_slice(raw_bytes),
    }
    .encode(buf)
    .unwrap();
}

pub(super) unsafe fn write_image(
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

    let raw_slice = unsafe { std::slice::from_raw_parts(data as *const u8, data_size) };

    let pitch = image_info.width as usize * image_info.channels as usize;
    let image = Image {
        pixels: raw_slice,
        width: image_info.width as usize,
        pitch: pitch,
        height: image_info.height as usize,
        format: pixel_format,
    };

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
