use super::super::schemas::{Bool, Float, Integer};
use super::helpers::data_as_ref;
use foxglove::Encode;
use foxglove::bytes::BufMut;
use std::ffi::c_void;
use std::mem;

// --- Writers ---

pub(super) unsafe fn write_int(buf: &mut impl BufMut, data: *const c_void, size: usize) {
    if size == mem::size_of::<i8>() {
        if let Some(value) = unsafe { data_as_ref::<i8>(data, size) } {
            Integer {
                value: *value as i32,
            }
            .encode(buf)
            .unwrap();
        }
    } else if size == mem::size_of::<i16>() {
        if let Some(value) = unsafe { data_as_ref::<i16>(data, size) } {
            Integer {
                value: *value as i32,
            }
            .encode(buf)
            .unwrap();
        }
    } else if size == mem::size_of::<i32>() {
        if let Some(value) = unsafe { data_as_ref::<i32>(data, size) } {
            Integer { value: *value }.encode(buf).unwrap();
        }
    }
}

pub(super) unsafe fn write_bool(buf: &mut impl BufMut, data: *const c_void, size: usize) {
    if let Some(value) = unsafe { data_as_ref::<bool>(data, size) } {
        Bool { value: *value }.encode(buf).unwrap();
    }
}

pub(super) unsafe fn write_float(buf: &mut impl BufMut, data: *const c_void, size: usize) {
    if let Some(value) = unsafe { data_as_ref::<f32>(data, size) } {
        Float { value: *value }.encode(buf).unwrap();
    }
}
