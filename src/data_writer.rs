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
use crate::types::{
    foxdbg_cube_t, foxdbg_line_t, foxdbg_pose_t, foxdbg_transform_t, foxdbg_vector3_t,
};

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

        unsafe {
            match channel_info.channel_type {
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_FLOAT => {
                    write_float(&mut *buf, data, size)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_INTEGER => {
                    write_int(&mut *buf, data, size)
                }
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_BOOLEAN => {
                    write_bool(&mut *buf, data, size)
                }
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
        }

        channel.log(&buf);
    });
}

unsafe fn write_location(buf: &mut impl BufMut, data: *const c_void, data_size: usize) {
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

unsafe fn write_transform(buf: &mut impl BufMut, data: *const c_void, data_size: usize) {
    if let Some(transform_data) = unsafe { data_as_ref::<foxdbg_transform_t>(data, data_size) } {
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

        let rotation = euler_to_quaternion(&transform_data.orientation, 0.0);

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
}

fn write_scene_update(
    buf: &mut impl BufMut,
    topic_name: &str,
    mutator: impl FnOnce(&mut SceneEntity),
) {
    let mut entity = SceneEntity {
        timestamp: None,
        frame_id: "world".to_owned(),
        id: topic_name.to_owned(),
        lifetime: None,
        frame_locked: false,
        metadata: Vec::new(),
        arrows: Vec::new(),
        cubes: Vec::new(),
        cylinders: Vec::new(),
        lines: Vec::new(),
        models: Vec::new(),
        spheres: Vec::new(),
        texts: Vec::new(),
        triangles: Vec::new(),
    };

    mutator(&mut entity);

    SceneUpdate {
        entities: vec![entity],
        deletions: Vec::new(),
    }
    .encode(buf)
    .unwrap();
}

unsafe fn write_lines(
    buf: &mut impl BufMut,
    data: *const c_void,
    data_size: usize,
    topic_name: &str,
) {
    let lines_slice = unsafe { data_as_slice::<foxdbg_line_t>(data, data_size) };
    if lines_slice.is_empty() {
        return;
    }

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

    write_scene_update(buf, topic_name, |entity| {
        entity.lines = line_primitives;
    });
}

unsafe fn write_pose(
    buf: &mut impl BufMut,
    data: *const c_void,
    data_size: usize,
    topic_name: &str,
) {
    if let Some(pose_data) = unsafe { data_as_ref::<foxdbg_pose_t>(data, data_size) } {
        let orientation = euler_to_quaternion(&pose_data.orientation, FRAC_PI_2);

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

        write_scene_update(buf, topic_name, |entity| {
            entity.arrows = vec![arrow];
        });
    }
}

unsafe fn write_cubes(
    buf: &mut impl BufMut,
    data: *const c_void,
    data_size: usize,
    topic_name: &str,
) {
    let cubes_slice = unsafe { data_as_slice::<foxdbg_cube_t>(data, data_size) };
    if cubes_slice.is_empty() {
        return;
    }

    let cube_primitives: Vec<CubePrimitive> = cubes_slice
        .iter()
        .map(|cube| {
            let position = Vector3 {
                x: cube.position.x as f64,
                y: cube.position.y as f64,
                z: cube.position.z as f64,
            };

            let orientation = euler_to_quaternion(&cube.orientation, FRAC_PI_2);

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

    write_scene_update(buf, topic_name, |entity| {
        entity.cubes = cube_primitives;
    });
}

fn euler_to_quaternion(orientation: &foxdbg_vector3_t, yaw_offset: f32) -> Quaternion {
    let pitch = orientation.x;
    let roll = orientation.y;
    let yaw = orientation.z + yaw_offset;

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

    Quaternion {
        x: qx as f64,
        y: qy as f64,
        z: qz as f64,
        w: qw as f64,
    }
}

unsafe fn write_pointcloud(buf: &mut impl BufMut, data: *const c_void, data_size: usize) {
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

unsafe fn write_image(
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

unsafe fn write_int(buf: &mut impl BufMut, data: *const c_void, size: usize) {
    if let Some(value) = unsafe { data_as_ref::<u32>(data, size) } {
        Integer { value: *value }.encode(buf).unwrap();
    }
}

unsafe fn write_bool(buf: &mut impl BufMut, data: *const c_void, size: usize) {
    if let Some(value) = unsafe { data_as_ref::<bool>(data, size) } {
        Bool { value: *value }.encode(buf).unwrap();
    }
}

unsafe fn write_float(buf: &mut impl BufMut, data: *const c_void, size: usize) {
    if let Some(value) = unsafe { data_as_ref::<f32>(data, size) } {
        Float { value: *value }.encode(buf).unwrap();
    }
}

pub unsafe fn write_channel_info(topic_name: &str, data: *const c_void, size: usize) {
    let mut channels = state::CHANNELS.lock().unwrap();
    let channel_state = channels
        .get_mut(topic_name)
        .expect("Channel info not found");

    let channel_info =
        if let Some(image_info) = unsafe { data_as_ref::<foxdbg_image_info_t>(data, size) } {
            match channel_state.channel_type {
                foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_IMAGE => {
                    ChannelInfo::CompressedImageInfo(*image_info)
                }
                _ => ChannelInfo::NoInfo(),
            }
        } else {
            ChannelInfo::NoInfo()
        };

    channel_state.channel_info = channel_info;
}

// --- Helpers ---

unsafe fn data_as_ref<'a, T>(data: *const c_void, size: usize) -> Option<&'a T> {
    if size != mem::size_of::<T>() {
        eprintln!(
            "Invalid data size for type {}, expected {} but got {}",
            std::any::type_name::<T>(),
            mem::size_of::<T>(),
            size
        );
        return None;
    }
    Some(unsafe { &*(data as *const T) })
}

unsafe fn data_as_slice<'a, T>(data: *const c_void, size: usize) -> &'a [T] {
    let item_size = mem::size_of::<T>();
    if item_size == 0 {
        return &[];
    }
    let num_items = size / item_size;
    if num_items == 0 {
        return &[];
    }
    unsafe { slice::from_raw_parts(data as *const T, num_items) }
}
