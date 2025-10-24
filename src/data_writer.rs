use foxglove::bytes::{BufMut, Bytes};
use foxglove::schemas::{
    ArrowPrimitive, Color, CompressedImage, CubePrimitive, FrameTransform, LinePrimitive,
    LocationFix, PackedElementField, Point3, PointCloud, Pose, Quaternion, SceneEntity,
    SceneUpdate, Timestamp, Vector3,
};
use foxglove::{Context, Encode};
use turbojpeg::{Compressor, Image, PixelFormat};

use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;
use std::ffi::{CStr, c_void};
use std::{mem, slice};

use crate::channel_schemas::{Bool, Float, Integer};
use crate::types::{foxdbg_cube_t, foxdbg_line_t, foxdbg_pose_t, foxdbg_transform_t};

use crate::state::{self, ChannelInfo};
use crate::{foxdbg_channel_type_t, foxdbg_image_info_t, foxdbg_location_t, foxdbg_vector4_t};

thread_local! {
    static LOG_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(1024 * 1024)); // 1MB initial capacity
}

pub fn write_channel(topic_name: &str, data: *const c_void, size: usize) {
    let channels = state::CHANNELS.lock().unwrap();
    let channel_info = channels.get(topic_name).expect("Channel info not found");

    let channel = Context::get_default()
        .get_channel_by_topic(topic_name)
        .expect("Channel could not be found");

    LOG_BUFFER.with(|buf_cell| {
        let mut buf = buf_cell.borrow_mut();
        buf.clear();

        match channel_info.channel_type {
            foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_FLOAT => write_float(&mut *buf, data),
            foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_INTEGER => write_int(&mut *buf, data),
            foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_BOOLEAN => write_bool(&mut *buf, data),
            foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_IMAGE => {
                write_image(&mut *buf, data, size, &channel_info.channel_info)
            }
            foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POINTCLOUD => {
                write_pointcloud(&mut *buf, data, size)
            }
            foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_CUBES => {
                write_cubes(&mut *buf, data, size, topic_name)
            }
            foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LINES => {
                write_lines(&mut *buf, data, size, topic_name)
            }
            foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POSE => {
                write_pose(&mut *buf, data, size, topic_name)
            }
            foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_TRANSFORM => {
                write_transform(&mut *buf, data, size)
            }
            foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LOCATION => {
                write_location(&mut *buf, data, size)
            }
        }

        channel.log(&buf);
    });
}

fn write_location(buf: &mut impl BufMut, data: *const c_void, data_size: usize) {
    if data_size != mem::size_of::<foxdbg_location_t>() {
        eprintln!("Invalid data size for location");
        return;
    }
    let location_data = unsafe { &*(data as *const foxdbg_location_t) };

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

fn write_transform(buf: &mut impl BufMut, data: *const c_void, data_size: usize) {
    if data_size != mem::size_of::<foxdbg_transform_t>() {
        eprintln!("Invalid data size for transform");
        return;
    }
    let transform_data = unsafe { &*(data as *const foxdbg_transform_t) };

    let parent_frame_id = unsafe {
        CStr::from_ptr(transform_data.parent_id)
            .to_string_lossy()
            .into_owned()
    };
    let child_frame_id = unsafe {
        CStr::from_ptr(transform_data.id)
            .to_string_lossy()
            .into_owned()
    };

    let translation = Vector3 {
        x: transform_data.position.x as f64,
        y: transform_data.position.y as f64,
        z: transform_data.position.z as f64,
    };

    // Euler to Quaternion conversion
    let pitch = transform_data.orientation.x;
    let roll = transform_data.orientation.y;
    let yaw = transform_data.orientation.z;

    let cy = (yaw * 0.5).cos();
    let sy = (yaw * 0.5).sin();
    let cp = (pitch * 0.5).cos();
    let sp = (pitch * 0.5).sin();
    let cr = (roll * 0.5).cos();
    let sr = (roll * 0.5).sin();

    let qx = sr * cp * cy - cr * sp * sy;
    let qy = cr * sp * cy + sr * cp * sy;
    let qz = cr * cp * sy - sr * sp * cy;
    let qw = cr * cp * cy + sr * sp * sy;

    let rotation = Quaternion {
        x: qx as f64,
        y: qy as f64,
        z: qz as f64,
        w: qw as f64,
    };

    FrameTransform {
        timestamp: None,
        parent_frame_id,
        child_frame_id,
        translation: Some(translation),
        rotation: Some(rotation),
    }
    .encode(buf)
    .unwrap();
}

fn write_lines(buf: &mut impl BufMut, data: *const c_void, data_size: usize, topic_name: &str) {
    let num_lines = data_size / mem::size_of::<foxdbg_line_t>();
    if num_lines == 0 {
        return;
    }
    let lines_slice = unsafe { slice::from_raw_parts(data as *const foxdbg_line_t, num_lines) };

    let line_primitives: Vec<LinePrimitive> = lines_slice
        .iter()
        .map(|line| {
            let start_point = Point3 {
                x: line.start.x as f64,
                y: line.start.y as f64,
                z: line.start.z as f64,
            };
            let end_point = Point3 {
                x: line.end.x as f64,
                y: line.end.y as f64,
                z: line.end.z as f64,
            };

            let pose = Pose {
                position: Some(Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
                orientation: Some(Quaternion {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                }),
            };

            let color = Color {
                r: line.color.r as f64,
                g: line.color.g as f64,
                b: line.color.b as f64,
                a: line.color.a as f64,
            };

            LinePrimitive {
                pose: Some(pose),
                thickness: line.thickness as f64,
                scale_invariant: false,
                points: vec![start_point, end_point],
                color: Some(color),
                colors: Vec::new(),
                indices: Vec::new(),
                r#type: 2,
            }
        })
        .collect();

    let entity = SceneEntity {
        timestamp: None,
        frame_id: "world".to_owned(),
        id: topic_name.to_owned(),
        lifetime: None,
        frame_locked: false,
        metadata: Vec::new(),
        arrows: Vec::new(),
        cubes: Vec::new(),
        cylinders: Vec::new(),
        lines: line_primitives,
        models: Vec::new(),
        spheres: Vec::new(),
        texts: Vec::new(),
        triangles: Vec::new(),
    };

    SceneUpdate {
        entities: vec![entity],
        deletions: Vec::new(),
    }
    .encode(buf)
    .unwrap();
}

fn write_pose(buf: &mut impl BufMut, data: *const c_void, data_size: usize, topic_name: &str) {
    if data_size != mem::size_of::<foxdbg_pose_t>() {
        eprintln!("Invalid data size for pose");
        return;
    }
    let pose_data = unsafe { &*(data as *const foxdbg_pose_t) };

    let pitch = pose_data.orientation.x;
    let roll = pose_data.orientation.y;
    let yaw = pose_data.orientation.z + FRAC_PI_2;

    let cy = (yaw * 0.5).cos();
    let sy = (yaw * 0.5).sin();
    let cp = (pitch * 0.5).cos();
    let sp = (pitch * 0.5).sin();
    let cr = (roll * 0.5).cos();
    let sr = (roll * 0.5).sin();

    let qx = sr * cp * cy - cr * sp * sy;
    let qy = cr * sp * cy + sr * cp * sy;
    let qz = cr * cp * sy - sr * sp * cy;
    let qw = cr * cp * cy + sr * sp * sy;

    let orientation = Quaternion {
        x: qx as f64,
        y: qy as f64,
        z: qz as f64,
        w: qw as f64,
    };

    let position = Vector3 {
        x: pose_data.position.x as f64,
        y: pose_data.position.y as f64,
        z: pose_data.position.z as f64,
    };

    let pose = Pose {
        position: Some(position),
        orientation: Some(orientation),
    };

    let color = Color {
        r: pose_data.color.r as f64,
        g: pose_data.color.g as f64,
        b: pose_data.color.b as f64,
        a: pose_data.color.a as f64,
    };

    let arrow = ArrowPrimitive {
        pose: Some(pose),
        shaft_length: 0.5,
        shaft_diameter: 0.05,
        head_length: 0.15,
        head_diameter: 0.1,
        color: Some(color),
    };

    let entity = SceneEntity {
        timestamp: None,
        frame_id: "world".to_owned(),
        id: topic_name.to_owned(),
        lifetime: None,
        frame_locked: false,
        metadata: Vec::new(),
        arrows: vec![arrow],
        cubes: Vec::new(),
        cylinders: Vec::new(),
        lines: Vec::new(),
        models: Vec::new(),
        spheres: Vec::new(),
        texts: Vec::new(),
        triangles: Vec::new(),
    };

    SceneUpdate {
        entities: vec![entity],
        deletions: Vec::new(),
    }
    .encode(buf)
    .unwrap();
}

fn write_cubes(buf: &mut impl BufMut, data: *const c_void, data_size: usize, topic_name: &str) {
    let num_cubes = data_size / mem::size_of::<foxdbg_cube_t>();
    if num_cubes == 0 {
        return;
    }
    let cubes_slice = unsafe { slice::from_raw_parts(data as *const foxdbg_cube_t, num_cubes) };

    let cube_primitives: Vec<CubePrimitive> = cubes_slice
        .iter()
        .map(|cube| {
            let position = Vector3 {
                x: cube.position.x as f64,
                y: cube.position.y as f64,
                z: cube.position.z as f64,
            };

            let pitch = cube.orientation.x;
            let roll = cube.orientation.y;
            let yaw = cube.orientation.z + FRAC_PI_2;

            let cy = (yaw * 0.5).cos();
            let sy = (yaw * 0.5).sin();
            let cp = (pitch * 0.5).cos();
            let sp = (pitch * 0.5).sin();
            let cr = (roll * 0.5).cos();
            let sr = (roll * 0.5).sin();

            let qx = sr * cp * cy - cr * sp * sy;
            let qy = cr * sp * cy + sr * cp * sy;
            let qz = cr * cp * sy - sr * sp * cy;
            let qw = cr * cp * cy + sr * sp * sy;

            let orientation = Quaternion {
                x: qx as f64,
                y: qy as f64,
                z: qz as f64,
                w: qw as f64,
            };

            let pose = Pose {
                position: Some(position),
                orientation: Some(orientation),
            };

            let size = Vector3 {
                x: cube.size.x as f64,
                y: cube.size.y as f64,
                z: cube.size.z as f64,
            };

            let colour = Color {
                r: cube.color.r as f64,
                g: cube.color.g as f64,
                b: cube.color.b as f64,
                a: cube.color.a as f64,
            };

            CubePrimitive {
                pose: Some(pose),
                size: Some(size),
                color: Some(colour),
            }
        })
        .collect();

    let entity = SceneEntity {
        timestamp: None,
        frame_id: "world".to_owned(),
        id: topic_name.to_owned(),
        lifetime: None,
        frame_locked: false,
        metadata: Vec::new(),
        arrows: Vec::new(),
        cubes: cube_primitives,
        cylinders: Vec::new(),
        lines: Vec::new(),
        models: Vec::new(),
        spheres: Vec::new(),
        texts: Vec::new(),
        triangles: Vec::new(),
    };

    SceneUpdate {
        entities: vec![entity],
        deletions: Vec::new(),
    }
    .encode(buf)
    .unwrap();
}

fn write_pointcloud(buf: &mut impl BufMut, data: *const c_void, data_size: usize) {
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
        timestamp: None, // TODO: Need to check if these are needed
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
