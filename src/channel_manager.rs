use foxglove::Channel;
use foxglove::schemas::{CompressedImage, FrameTransform, LocationFix, PointCloud, SceneUpdate};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use crate::channel_schemas::{Bool, Float, Integer};
use crate::foxdbg_channel_type_t;

use crate::state;

pub fn add_channel(topic_name: &str, channel_type: foxdbg_channel_type_t, target_hz: c_int) {
    match channel_type {
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_FLOAT => {
            Channel::<Float>::new(topic_name);
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_IMAGE => {
            Channel::<CompressedImage>::new(topic_name);
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POINTCLOUD => {
            Channel::<PointCloud>::new(topic_name);
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_CUBES
        | foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LINES
        | foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_POSE => {
            Channel::<SceneUpdate>::new(topic_name);
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_TRANSFORM => {
            Channel::<FrameTransform>::new(topic_name);
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_LOCATION => {
            Channel::<LocationFix>::new(topic_name);
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_INTEGER => {
            Channel::<Integer>::new(topic_name);
        }
        foxdbg_channel_type_t::FOXDBG_CHANNEL_TYPE_BOOLEAN => {
            Channel::<Bool>::new(topic_name);
        }
    };

    let mut channels = state::CHANNELS.lock().unwrap();
    channels.insert(topic_name.to_string(), channel_type);
}

pub fn add_rx_channel(topic_name: &str, channel_type: foxdbg_channel_type_t) -> c_int {
    println!(
        "channel_manager::add_rx_channel called with topic: {}, type: {:?}",
        topic_name, channel_type
    );
    // Placeholder: return a dummy channel ID
    2
}

pub fn get_rx_channel(topic_name: *const c_char) -> c_int {
    let c_str = unsafe { CStr::from_ptr(topic_name) };
    let topic_name_str = c_str.to_str().unwrap_or("invalid_topic");
    println!(
        "channel_manager::get_rx_channel called with topic: {}",
        topic_name_str
    );
    // Placeholder: return a dummy channel ID
    2
}
