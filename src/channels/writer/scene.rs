use foxglove::Encode;
use foxglove::bytes::BufMut;
use foxglove::schemas::{
    ArrowPrimitive, Color, CubePrimitive, FrameTransform, LinePrimitive, Point3, Pose, Quaternion,
    SceneEntity, SceneUpdate, Vector3,
};
use std::f32::consts::FRAC_PI_2;
use std::ffi::{CStr, c_void};

use crate::types::{foxdbg_cube_t, foxdbg_line_t, foxdbg_pose_t, foxdbg_transform_t};

use super::helpers::{data_as_ref, data_as_slice, euler_to_quaternion};

/// A generic helper function for creating and encoding `SceneUpdate` messages.
///
/// This function simplifies the process of sending scene updates to Foxglove by handling
/// the boilerplate of creating a `SceneEntity` and a `SceneUpdate`. It takes a `mutator`
/// closure that can be used to customize the `SceneEntity` before it's encoded.
///
/// # Arguments
///
/// * `buf` - The buffer to write the encoded `SceneUpdate` to.
/// * `topic_name` - The name of the topic to associate with the `SceneEntity`.
/// * `mutator` - A closure that takes a mutable reference to a `SceneEntity` and
///   modifies it as needed.
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

// --- Writers ---

pub(super) unsafe fn write_transform(buf: &mut impl BufMut, data: *const c_void, data_size: usize) {
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

pub(super) unsafe fn write_lines(
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

pub(super) unsafe fn write_pose(
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

pub(super) unsafe fn write_cubes(
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
