/// This file defines custom message schemas to be used by channels
use foxglove::Encode;

use crate::foxdbg_image_info_t;

#[derive(Encode)]
pub struct Float {
    pub value: f32
}

#[derive(Encode)]
pub struct Integer {
    pub value: u32
}

#[derive(Encode)]
pub struct Bool {
    pub value: bool
}

pub enum ChannelInfo {
    CompressedImageInfo(foxdbg_image_info_t),
    NoInfo()
}
