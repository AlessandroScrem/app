use std::collections::HashMap;

use crate::assets::material_desc::MaterialDesc;
use crate::ecs::components::*;

use crate::assets::MaterialId;
use crate::assets::asset_manager::{AssetManager, GlobalAssetId, ResourceStats};
use crate::assets::MeshAsset;
use crate::assets::MaterialAsset;
use crate::gpu::GpuInternalCounters;
use crate::ui::UiTexture;
use crate::ui::traits::UiTextureResolver;
use crate::renderer::scene_renderer::FrameStats;
use crate::Globals;
use crate::Camera;
use crate::assets::TextureId;

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
    pub frame: FrameStats,
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
    pub gpu_counters: GpuInternalCounters,
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

impl UiComponentState {
    pub fn from_world(selected: Option<Entity>, world: &World, asset_mgr: &AssetManager) -> Self {
        let mut state = UiComponentState::default();

        let Some(entity) = selected else {
            return state;
        };
        let Ok(entry) = world.entry_ref(entity) else {
            return state;
        };

        state.tag = entry.get_component::<TagComponent>().ok().cloned();
        state.transform = entry.get_component::<TransformComponent>().ok().cloned();
        state.bounding_box = entry.get_component::<BoundingBoxComponent>().ok().cloned();
        state.light = entry.get_component::<LightComponent>().ok().cloned();

        if let Ok(mesh) = entry.get_component::<MeshComponent>() {
            state.mesh = Some(mesh.clone());


            if let Some(mesh_asset) = asset_mgr.get::<MeshAsset>(mesh.handle) {
                let materials: HashMap<MaterialId, MaterialDesc> = mesh_asset.desc
                    .submeshes
                    .iter()
                    .filter_map(|sm| {
                        asset_mgr
                            .get::<MaterialAsset>(sm.material)
                            .map(|mat_asset| (sm.material, mat_asset.desc.clone()))
                    })
                    .collect();

                state.materials = Some(materials);
            }
        }

        state
    }
}

impl<'a> UiSnapshot<'a> {
    pub fn from_world(
        world: &legion::World,
        selected: Option<Entity>,
        asset_mgr: &AssetManager,
        camera: &'a Camera,
        globals: &'a Globals,
        texture_resolver: &'a dyn UiTextureResolver,
        gpu_counters: GpuInternalCounters,
        debug_texture_id: Option<UiTexture>,
        render_stats: RenderStats,
        hdr_id: Option<TextureId>,
        scene_name: Option<String>,
    ) -> Self {
        let root_snapshot = RootSnapshot {
            root_nodes: get_hierarchy_roots(world),
            lights_nodes: get_lights_nodes(world),
        };

        let comp_state = UiComponentState::from_world(selected, world, asset_mgr);

        Self {
            texture_resolver,
            camera,
            globals,
            root_snapshot,
            comp_state,
            selected,
            hovered: None,
            debug_texture_id,
            gpu_counters,
            render_stats,
            hdr_id,
            scene_name
        }
    }
}

/// Costruisce la gerarchia degli oggetti
fn get_hierarchy_roots(world: &legion::World) -> RootNodes {
    let mut roots = RootNodes::default();
    let mut query = <(Entity, &HierarchyComponent)>::query();

    for (entity, hierarchy) in query.iter(world) {
        if hierarchy.parent.is_none() {
            roots.nodes.push(build_node(world, *entity, None));
        }
    }

    roots
}

/// Costruisce i nodi root per le luci
fn get_lights_nodes(world: &legion::World) -> LightNodes {
    let mut roots = LightNodes::default();
    let mut query = <(Entity, &LightComponent, &TagComponent)>::query();

    for (entity, light, tag) in query.iter(world) {
        roots.nodes.push(LightNode {
            name: tag.name.clone(),
            comp: light.clone(), 
            entity: *entity,
        });
    }

    roots
}

/// Funzione ricorsiva che costruisce un nodo con tutti i figli
fn build_node(world: &legion::World, entity: Entity, parent: Option<Entity>) -> HierarchyNode {
    let entry = match world.entry_ref(entity) {
        Ok(e) => e,
        Err(_) => {
            return HierarchyNode {
                name: "<missing>".into(),
                parent,
                entity,
                children: Vec::new(),
            };
        }
    };

    let name = entry
        .get_component::<TagComponent>()
        .map(|t| t.name.clone())
        .unwrap_or_else(|_| "<unnamed>".into());

    let children = entry
        .get_component::<HierarchyComponent>()
        .map(|h| {
            h.children
                .iter()
                .map(|&child| build_node(world, child, Some(entity)))
                .collect()
        })
        .unwrap_or_default();

    HierarchyNode {
        name,
        parent,
        entity,
        children,
    }
}
