use std::collections::HashMap;

use super::*;
use crate::assets::MaterialId;
use crate::assets::global_asset_manager::GlobalAssetId;
use crate::assets::global_asset_manager::resource_stats::ResourceStats;
use crate::gpu::caches::internalcounter::GpuInternalCounters;
use crate::prelude::*;
use crate::ui::traits::UiTextureResolver;
use legion::*;

pub struct HierarchyNode {
    pub name: String,
    pub parent: Option<Entity>,
    pub entity: Entity,
    pub children: Vec<HierarchyNode>,
}

#[derive(Default)]
pub struct RootNodes {
    pub nodes: Vec<HierarchyNode>,
}

#[derive(Default)]
pub struct RootSnapshot {
    pub root_nodes: RootNodes,
    pub lights_nodes: RootNodes,
}

#[derive(Default)]
pub struct RenderStats {
    pub texture: ResourceStats,
    pub mesh: ResourceStats,
    pub material: ResourceStats,
    pub frame: renderer::scene_renderer::FrameStats,
}

pub struct UiSnapshot<'a> {
    pub texture_resolver: &'a dyn UiTextureResolver,
    pub camera: &'a Camera,
    pub globals: &'a Globals,
    pub root_snapshot: RootSnapshot,
    pub comp_state: UiComponentState,
    pub selected: Option<Entity>,
    pub hovered: Option<Entity>,
    pub debug_texture_id: Option<assets::TextureId>,
    pub render_stats: RenderStats,
    pub gpu_counters: GpuInternalCounters,
    pub hdr_id: Option<GlobalAssetId>,
}

/// UiComponentView is a per-frame snapshot.
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
    pub fn from_world(selected: Option<Entity>, world: &World, asset_mgr: &GlobalAssetManager) -> Self {
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
            use crate::assets::mesh_asset::MeshAsset;
            use crate::assets::material_asset::MaterialAsset;

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
        asset_mgr: &GlobalAssetManager,
        camera: &'a Camera,
        globals: &'a Globals,
        texture_resolver: &'a dyn UiTextureResolver,
        gpu_counters: GpuInternalCounters,
        debug_texture_id: Option<assets::TextureId>,
        render_stats: RenderStats,
        hdr_id: Option<assets::TextureId>,
    ) -> Self {
        let root_snapshot = RootSnapshot {
            root_nodes: get_hierarchy_roots(world),
            lights_nodes: get_lights_roots(world),
        };

        let comp_state = UiComponentState::from_world(selected, world, asset_mgr);
        // let hdr_texture_id = asset_mgr.skybox.get_id();
        // let hdr_texture_id = GlobalAssetId::default();
        

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
fn get_lights_roots(world: &legion::World) -> RootNodes {
    let mut roots = RootNodes::default();
    let mut query = <(Entity, &LightComponent, &TagComponent)>::query();

    for (entity, _light, tag) in query.iter(world) {
        roots.nodes.push(HierarchyNode {
            name: tag.name.clone(),
            parent: None,
            entity: *entity,
            children: Vec::new(),
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
