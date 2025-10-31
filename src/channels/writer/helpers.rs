use std::ffi::c_void;
use std::{mem, slice};

use foxglove::schemas::Quaternion;

use crate::foxdbg_vector3_t;

pub(super) unsafe fn data_as_ref<'a, T>(data: *const c_void, size: usize) -> Option<&'a T> {
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

pub(super) unsafe fn data_as_slice<'a, T>(data: *const c_void, size: usize) -> &'a [T] {
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

pub(super) fn euler_to_quaternion(orientation: &foxdbg_vector3_t, yaw_offset: f32) -> Quaternion {
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
