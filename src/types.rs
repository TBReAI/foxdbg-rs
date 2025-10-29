#![allow(non_camel_case_types)]
use std::os::raw::c_char;

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

/// Represents a transform between two coordinate frames.
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
