use std::collections::HashMap;

use legion::Entity;

use super::*;

use crate::assets::asset_manager::AssetManager;
use crate::assets::{
    LinesVertexData, MaterialAsset, MaterialId, MeshAsset, MeshId, VertexInstance,
};

use crate::engine::gpu_sync::TextureLoadService;
use crate::globals::Globals;
use crate::gpu::GpuCache;

use crate::prelude::trace;
use crate::renderer::line_builder::{
    AxisAlignedBoundingBox, LineDrawable, ObjectOrientedBoundingBox,
};
use crate::renderer::render_objects::{
    BboxRenderObject, LightRenderObject, MeshRenderObject, RenderObjects,
};
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

    pub light_uniform: LightsUniform,

    pub opaque_stats: DrawStats,
    pub transmission_stats: DrawStats,
}

impl FrameBuilder {
    pub fn prepare(
        objects: &RenderObjects,
        assets: &AssetManager,
        globals: &Globals,
        gpu_cache: &GpuCache,
        texture_loader: &TextureLoadService,
    ) -> Self {
        let mut frame = FrameBuilder::default();

        // create batches & instances for meshes
        Self::prepare_meshes(
            &objects.meshes,
            assets,
            gpu_cache,
            texture_loader,
            &mut frame,
        );

        // create uniform for lights
        Self::prepare_light_uniform(&objects.lights, &mut frame, globals.light_enable);

        // crate lines vertexdata.
        Self::extract_bbox_lines(&objects.bboxes, &mut frame.lines, globals.bbox_axis_aligned);
        Self::extract_light_frustums(&objects.lights, &mut frame.lines);

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

    fn extract_bbox_lines(
        bbox: &[BboxRenderObject],
        lines: &mut Vec<LinesVertexData>,
        axis_aligned: bool,
    ) {
        for b in bbox {
            if axis_aligned {
                AxisAlignedBoundingBox {
                    bbox: &b.bbox.global_bounding_box,
                }
                .emit(lines);
            } else {
                ObjectOrientedBoundingBox {
                    bbox: &b.bbox.bounding_box,
                    transform: &b.transform,
                }
                .emit(lines);
            }
        }
    }

    fn extract_light_frustums(lights: &[LightRenderObject], lines: &mut Vec<LinesVertexData>) {
        lights
            .iter()
            .filter(|l| l.light.frustum)
            .take(uniform::MAX_LIGHTS)
            .for_each(|l| l.emit(lines));
    }

    fn prepare_meshes(
        meshes: &[MeshRenderObject],
        assets: &AssetManager,
        gpu_cache: &GpuCache,
        texture_loader: &TextureLoadService,
        frame: &mut FrameBuilder,
    ) {
        let mut opaque: HashMap<BatchKey, Vec<VertexInstance>> = HashMap::new();
        let mut transmission: HashMap<BatchKey, Vec<VertexInstance>> = HashMap::new();

        for object in meshes {
            // Publish a whole object only when its mesh and every submesh material
            // are ready. Failed textures count as resolved because their material
            // bind groups use the built-in white fallback.
            if gpu_cache.mesh.get(&object.mesh).is_none() {
                continue;
            }
            let Some(mesh) = assets.get::<MeshAsset>(object.mesh) else {
                continue;
            };
            let materials_ready = mesh.desc.submeshes.iter().all(|submesh| {
                assets
                    .get::<MaterialAsset>(submesh.material)
                    .is_some_and(|material| {
                        gpu_cache.material.get(&submesh.material).is_some()
                            && texture_loader.is_material_ready(&material.desc)
                    })
            });
            if !materials_ready {
                continue;
            }

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
        let mut ordered = map.into_iter().collect::<Vec<_>>();
        ordered.sort_by_key(|(key, _)| {
            (
                key.mesh.raw(),
                key.material.raw(),
                key.index_start,
                key.index_end,
            )
        });

        for (key, batch_instances) in ordered {
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

    fn prepare_light_uniform(
        lights_objects: &[LightRenderObject],
        frame: &mut FrameBuilder,
        enabled: bool,
    ) {
        let lights_uniform = &mut frame.light_uniform;

        *lights_uniform = LightsUniform::default();
        lights_uniform.enabled = enabled.into();

        lights_objects
            .iter()
            .take(uniform::MAX_LIGHTS)
            .enumerate()
            .for_each(|(i, light_object)| {
                lights_uniform.count = (i + 1) as u32;
                lights_uniform.lights[i] = LightUniform::from(light_object);
            });
    }
}

#[cfg(test)]
mod tests {
    use super::{BatchKey, FrameBuilder, InstanceBatch};
    use crate::assets::{MaterialId, MeshId, VertexInstance};
    use std::collections::HashMap;

    #[test]
    fn batches_are_sorted_deterministically() {
        let mesh_a = MeshId::new();
        let mesh_b = MeshId::new();
        let material = MaterialId::new();
        let mut map = HashMap::new();

        map.insert(
            BatchKey {
                mesh: mesh_b,
                material,
                index_start: 0,
                index_end: 3,
            },
            Vec::<VertexInstance>::new(),
        );
        map.insert(
            BatchKey {
                mesh: mesh_a,
                material,
                index_start: 0,
                index_end: 3,
            },
            Vec::<VertexInstance>::new(),
        );

        let mut batches: Vec<InstanceBatch> = Vec::new();
        let mut instances = Vec::new();
        FrameBuilder::flush_batches(map, &mut batches, &mut instances);

        assert_eq!(
            batches
                .iter()
                .map(|batch| batch.mesh.raw())
                .collect::<Vec<_>>(),
            vec![mesh_a.raw(), mesh_b.raw()]
        );
    }
}
