use std::ffi::c_void;
use std::{mem, slice};

use foxglove::schemas::Quaternion;

use crate::foxdbg_vector3_t;

/// Converts a raw C pointer to a Rust reference.
///
/// This function takes a raw C pointer (`data`) and a `size`, and attempts to
/// convert it into a Rust reference of type `T`. It checks that the `size` matches
/// the size of `T` to prevent memory access errors. If the sizes do not match, it
/// logs a warning and returns `None`.
///
/// # Safety
///
/// This function is `unsafe` because it dereferences a raw pointer. The caller must
/// ensure that the pointer is valid, non-null, and points to a valid instance of `T`.
///
/// # Arguments
///
/// * `data` - A raw C pointer to the data.
/// * `size` - The size of the data in bytes.
///
/// # Returns
///
/// An `Option` containing a reference to the data if the size is valid, or `None`
/// otherwise.
pub(super) unsafe fn data_as_ref<'a, T>(data: *const c_void, size: usize) -> Option<&'a T> {
    if size != mem::size_of::<T>() {
        log::warn!(
            "Invalid data size for type {}, expected {} but got {}",
            std::any::type_name::<T>(),
            mem::size_of::<T>(),
            size
        );
        return None;
    }
    Some(unsafe { &*(data as *const T) })
}

/// Converts a raw C pointer to a Rust slice.
///
/// This function takes a raw C pointer (`data`) and a total `size` in bytes, and
/// converts it into a Rust slice of type `T`. It calculates the number of items in
/// the slice based on the size of `T`.
///
/// # Safety
///
/// This function is `unsafe` because it dereferences a raw pointer and creates a
/// slice from it. The caller must ensure that the pointer is valid, non-null, and
/// that the `size` correctly corresponds to a valid sequence of `T` instances in
/// memory.
///
/// # Arguments
///
/// * `data` - A raw C pointer to the data.
/// * `size` - The total size of the data in bytes.
///
/// # Returns
///
/// A slice of type `T`.
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

/// Converts Euler angles to a quaternion, with an optional yaw offset.
///
/// This function assumes the input Euler angles (`orientation`) are in a right-handed
/// coordinate system, where X is pitch, Y is roll, and Z is yaw. The `yaw_offset` is
/// added to the yaw before conversion, which is useful for aligning coordinate frames.
/// For example, a `FRAC_PI_2` offset can be used to rotate the coordinate system by
/// 90 degrees around the Z-axis.
///
/// # Arguments
///
/// * `orientation` - A reference to a `foxdbg_vector3_t` containing the Euler angles
///   (pitch, roll, yaw).
/// * `yaw_offset` - An offset to be added to the yaw angle, in radians.
///
/// # Returns
///
/// A `Quaternion` representing the converted orientation.
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
