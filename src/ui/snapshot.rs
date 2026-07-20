use std::collections::HashMap;

use crate::assets::material_desc::MaterialDesc;
use crate::ecs::components::*;
use crate::assets::MaterialId;
use crate::assets::asset_manager::{GlobalAssetId, ResourceStats};
use crate::gpu::GpuInternalCounters;
use crate::ui::UiTexture;
use crate::ui::traits::UiTextureResolver;
use crate::renderer::scene_renderer::FrameStats;
use crate::Globals;
use crate::Camera;

use legion::*;


pub struct HierarchyNode {
    pub name: String,
    pub parent: Option<Entity>,
    pub entity: Entity,
    pub children: Vec<HierarchyNode>,
}

pub struct LightNode {
    pub name: String,
    pub comp: LightComponent,
    pub entity: Entity,
}

#[derive(Default)]
pub struct RootNodes {
    pub nodes: Vec<HierarchyNode>,
}

#[derive(Default)]
pub struct LightNodes {
    pub nodes: Vec<LightNode>,
}

#[derive(Default)]
pub struct RootSnapshot {
    pub root_nodes: RootNodes,
    pub lights_nodes: LightNodes,
}

#[derive(Default)]
pub struct RenderStats {
    pub texture: ResourceStats,
    pub mesh: ResourceStats,
    pub material: ResourceStats,
    pub gpu_counters: GpuInternalCounters,
    pub frame_stats: FrameStats,
}

pub struct UiSnapshot<'a> {
    pub texture_resolver: &'a dyn UiTextureResolver,
    pub camera: &'a Camera,
    pub globals: &'a Globals,
    pub root_snapshot: RootSnapshot,
    pub comp_state: UiComponentState,
    pub selected: Option<Entity>,
    pub hovered: Option<Entity>,
    pub debug_texture_id: Option<UiTexture>,
    pub render_stats: RenderStats,
    pub hdr_id: Option<GlobalAssetId>,
    pub scene_name: Option<String>,
}

/// UiComponentState is a per-frame snapshot.
/// It must never be stored or reused across frames.
#[derive(Default)]
pub struct UiComponentState {
    pub tag: Option<TagComponent>,
    pub mesh: Option<MeshComponent>,
    pub transform: Option<TransformComponent>,
    pub bounding_box: Option<BoundingBoxComponent>,
    pub materials: Option<HashMap<MaterialId, MaterialDesc>>,
    pub light: Option<LightComponent>,
}
