/// Writer functions for telemetary data
use super::super::schemas::{Bool, Float, Integer};
use super::helpers::data_as_ref;
use foxglove::Encode;
use foxglove::bytes::BufMut;
use std::ffi::c_void;
use std::mem::size_of;

const I8_SIZE: usize = size_of::<i8>();
const I16_SIZE: usize = size_of::<i16>();
const I32_SIZE: usize = size_of::<i32>();

unsafe fn encode_integer<T: Into<i32> + Copy>(
    buf: &mut impl BufMut,
    data: *const c_void,
    size: usize,
) {
    if let Some(value) = unsafe { data_as_ref::<T>(data, size) } {
        Integer {
            value: (*value).into(),
        }
        .encode(buf)
        .unwrap();
    }
}

pub(super) unsafe fn write_int(buf: &mut impl BufMut, data: *const c_void, size: usize) {
    // This can be called with multiple different int types so need to check which one
    match size {
        I8_SIZE => unsafe { encode_integer::<i8>(buf, data, size) },
        I16_SIZE => unsafe { encode_integer::<i16>(buf, data, size) },
        I32_SIZE => unsafe { encode_integer::<i32>(buf, data, size) },
        _ => {}
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
