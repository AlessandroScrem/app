use std::collections::HashMap;

use legion::Entity;

use super::*;

use crate::assets::asset_manager::AssetManager;
use crate::assets::{
    LinesVertexData, MaterialAsset, MaterialId, MeshAsset, MeshId, VertexInstance,
};

use crate::globals::Globals;

use crate::prelude::trace;
use crate::renderer::render_queue::RenderQueue;
use crate::renderer::uniform::{LightUniform, LightsUniform};



pub struct InstanceBatch {
    pub mesh: MeshId,
    pub material: MaterialId,

    pub submesh_index_range: std::ops::Range<u32>,

    pub instance_start: u32,
    pub instance_count: u32,
}

#[derive(Hash, PartialEq, Eq)]
struct BatchKey {
    mesh: MeshId,
    material: MaterialId,

    index_start: u32,
    index_end: u32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct DrawStats {
    pub draw_calls: u32,
    pub instances: u32,
}

#[derive(Default)]
pub struct FrameTasks {
    pub axis_enable: bool,
    pub entity_selected: Option<Entity>,
    pub skybox_enable: bool,
    pub skybox_blur: bool,
    pub build_mips_cp: bool,
}

pub struct FrameData {
    // geometry
    pub opaque_batches: Vec<InstanceBatch>,
    pub transmission_batches: Vec<InstanceBatch>,
    pub lines: Vec<LinesVertexData>,

    // runtime data
    pub lights: Option<LightsUniform>,

    // flags / tasks
    pub tasks: FrameTasks,

    // stats
    pub opaque_stats: DrawStats,
    pub transmission_stats: DrawStats,
}

fn compute_stats(batches: &[InstanceBatch]) -> DrawStats {
    let draw_calls = batches.len() as u32;

    let instances = batches.iter().map(|b| b.instance_count).sum::<u32>();
    DrawStats {
        draw_calls,
        instances,
    }
}

#[derive(Default)]
pub struct FrameBuilder {
    pub opaque_batches: Vec<InstanceBatch>,
    pub transmission_batches: Vec<InstanceBatch>,

    pub instances: Vec<VertexInstance>,
    pub lines: Vec<LinesVertexData>,

    pub lights: LightsUniform,

    pub opaque_stats: DrawStats,
    pub transmission_stats: DrawStats,
}

impl FrameBuilder {
    pub fn prepare(queue: RenderQueue, assets: &AssetManager, globals: &Globals) -> Self {
        let mut frame = FrameBuilder::default();

        // create batches & instances for meshes
        Self::prepare_meshes(&queue, assets, &mut frame);
        // create uniform for lights
        Self::prepare_lights(&queue, globals, &mut frame);
        // move lines vertexdata created in previus pass.
        // TODO: to be implemented here
        Self::prepare_lines(queue, &mut frame);

        // Calc Frame stats:
        // TODO: move away from here
        frame.opaque_stats = compute_stats(&frame.opaque_batches);
        frame.transmission_stats = compute_stats(&frame.transmission_batches);

        trace!(
            "Opaque Stats: {:?}, Transmission Stats: {:?}, Total DrawCall: {}",
            frame.opaque_stats,
            frame.transmission_stats,
            frame.opaque_stats.draw_calls + frame.transmission_stats.draw_calls
        );

        frame
    }

    fn prepare_lines(queue: RenderQueue, frame: &mut FrameBuilder) {
        frame.lines = queue.lines
    }

    fn prepare_meshes(queue: &RenderQueue, assets: &AssetManager, frame: &mut FrameBuilder) {
        let mut opaque: HashMap<BatchKey, Vec<VertexInstance>> = HashMap::new();
        let mut transmission: HashMap<BatchKey, Vec<VertexInstance>> = HashMap::new();

        for object in &queue.mesh {
            let Some(mesh) = assets.get::<MeshAsset>(object.mesh) else {
                continue;
            };

            for submesh in &mesh.desc.submeshes {
                let Some(material) = assets.get::<MaterialAsset>(submesh.material) else {
                    continue;
                };

                let key = BatchKey {
                    mesh: object.mesh,
                    material: submesh.material,
                    index_start: submesh.index_range.start,
                    index_end: submesh.index_range.end,
                };

                let instance = VertexInstance::new(object.transform, object.entity_id);

                if material.desc.is_transmissive() {
                    transmission.entry(key).or_default().push(instance);
                } else if !material.desc.is_transparent() {
                    opaque.entry(key).or_default().push(instance);
                }
            }
        }

        FrameBuilder::flush_batches(opaque, &mut frame.opaque_batches, &mut frame.instances);

        FrameBuilder::flush_batches(
            transmission,
            &mut frame.transmission_batches,
            &mut frame.instances,
        );
    }

    // ---------------------------------------------------------
    // BUILD FINAL BATCHES
    // ---------------------------------------------------------
    fn flush_batches(
        map: HashMap<BatchKey, Vec<VertexInstance>>,
        batches: &mut Vec<InstanceBatch>,
        instances: &mut Vec<VertexInstance>,
    ) {
        for (key, batch_instances) in map {
            let start = instances.len() as u32;
            let count = batch_instances.len() as u32;

            instances.extend(batch_instances);

            batches.push(InstanceBatch {
                mesh: key.mesh,
                material: key.material,

                submesh_index_range: key.index_start..key.index_end,

                instance_start: start,
                instance_count: count,
            });
        }
    }

    fn prepare_lights(queue: &RenderQueue, globals: &Globals, frame: &mut FrameBuilder) {
        let lights_uniform = &mut frame.lights;

        *lights_uniform = LightsUniform::default();
        lights_uniform.enabled = globals.light_enable.into();

        for (i, light) in queue
            .lights
            .iter()
            .filter(|l| l.light.enabled)
            .take(uniform::MAX_LIGHTS)
            .enumerate()
        {
            let uniform = LightUniform {
                entity_id: light.entity_id,
                ..(&light.light).into()
            };
            lights_uniform.count = (i + 1) as u32;
            lights_uniform.lights[i] = uniform;
        }
    }

}
