use foxglove::schemas::{CompressedImage, FrameTransform, LocationFix, PointCloud, SceneUpdate};
use foxglove::{Channel, Context};

use super::schemas::{Bool, Float, Integer};
use crate::foxdbg_channel_type_t;

use crate::state::{self, ChannelInfo, ChannelState};

/// Creates a new channel for publishing data to Foxglove.
///
/// This function takes a topic name and a channel type, creates a new Foxglove channel
/// with the appropriate schema, and stores the channel's state in the global `CHANNELS`
/// map. It returns a channel ID that can be used to publish data to the channel.
///
/// # Arguments
///
/// * `topic_name` - The name of the topic to create.
/// * `channel_type` - The type of data that will be published on the channel.
///
/// # Returns
///
/// The ID of the newly created channel.
pub fn add_channel(
    topic_name: &str,
    channel_type: foxdbg_channel_type_t,
) -> u64 {
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

    let channel_id = Context::get_default()
        .get_channel_by_topic(topic_name)
        .unwrap()
        .id();

    let mut channels = state::CHANNELS.lock().unwrap();
    let state = ChannelState {
        channel_type: channel_type,
        channel_info: ChannelInfo::NoInfo(),
        channel_topic: topic_name.to_owned() 
    };

    channels.insert(channel_id, state);
    channel_id.into()
}
