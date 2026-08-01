use std::{fs, path::Path};

use legion::*;
use serde::{Deserialize, Serialize};

use crate::{
    app::domain::events::{DomainEvent, SceneEvent}, assets::{
        MeshAsset,
        asset_manager::AssetManager,
        gltf_loader::{LoadedScene, NodeData, load_gltf},
    }, ecs::components::*, engine::{RuntimeEvent, engine::EventBus}, ui::UiComponentState,
};

#[derive(Serialize, Deserialize)]
pub struct SceneFile {
    pub version: u32,
    pub scenes: Vec<SceneEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct SceneEntry {
    pub path: String,

    pub transform: TransformComponent,
}

pub struct Scene {
    pub filename: Option<String>,
    pub world: World,
    pub resources: Resources,
    pub schedule: Schedule,
    pub dirty: bool,
}

impl Default for Scene {
    fn default() -> Self {
        let world = World::default();
        let schedule = crate::ecs::create_current_scene_schedule_builder();
        let resources = Resources::default();

        Scene {
            filename: None,
            world,
            resources,
            schedule,
            dirty: true,
        }
    }
}

impl Scene {
    pub fn update_scene(&mut self, bus: &mut EventBus) {
        self.schedule.execute(&mut self.world, &mut self.resources);

        if self.dirty {
            let title = self.filename.clone().unwrap_or("Untitled scene *".into());
            bus.send_runtime(RuntimeEvent::SetWindowTitle(title));
            self.dirty = false;
        }
    }
}

impl Scene {
    pub fn clear_scene(&mut self, asset_mgr: &mut AssetManager) {
        for entity in hierarchy::collect_hierarchy_root_entities(&self.world).iter() {
            hierarchy::remove_entity(asset_mgr, *entity, &mut self.world);
        }
        self.filename = None;
        self.dirty = true;
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        let filename = self
            .filename
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Nessun file scena aperto"))?;
        self.save_scene_json(filename)
    }

    pub fn save_scene_json<P: AsRef<Path>>(&mut self, filename: P) -> anyhow::Result<()> {
        let mut scenes = Vec::new();

        let mut query = <(&SceneComponent, &TransformComponent)>::query();

        for (scene, transform) in query.iter(&self.world) {
            scenes.push(SceneEntry {
                path: scene.path.clone(),
                transform: transform.clone(),
            });
        }

        let file = SceneFile { version: 1, scenes };

        let json = serde_json::to_string_pretty(&file)?;

        fs::write(&filename, json)?;

        let string_name = filename.as_ref().to_string_lossy().to_string();
        self.filename = Some(string_name);
        self.dirty = true;

        Ok(())
    }

    pub fn open_scene<P: AsRef<Path>>(
        &mut self,
        filename: P,
        asset_mgr: &mut AssetManager,
        bus: &mut EventBus,
    ) -> anyhow::Result<()> {
        // 1. leggi file scena
        let json = fs::read_to_string(&filename)?;

        // 2. deserialize
        let scene_file: SceneFile = serde_json::from_str(&json)?;

        // 3. carica tutte le scene presenti
        for scene in scene_file.scenes {
            let pathname = std::path::PathBuf::from(&scene.path);

            // carica il gltf
            let loaded = load_gltf(pathname, asset_mgr);

            // 4. ricrea trasformazione root
            let root_transform = scene.transform.clone();

            if let Some(loaded) = loaded {
                // 5. ricrea entity ECS
                bus.send_domain(DomainEvent::Scene(SceneEvent::AddComponent(
                    loaded,
                    root_transform,
                )));
            }
        }

        let string_name = filename.as_ref().to_string_lossy().to_string();
        self.filename = Some(string_name);
        self.dirty = true;

        Ok(())
    }
}

fn collect_mesh_nodes(nodes: &[NodeData], index: usize, output: &mut Vec<usize>) -> bool {
    let node = &nodes[index];

    let mut has_mesh_child = false;

    for &child in &node.children {
        if collect_mesh_nodes(nodes, child, output) {
            has_mesh_child = true;
        }
    }

    if node.mesh.is_some() || has_mesh_child {
        output.push(index);
        true
    } else {
        false
    }
}

pub fn spawn_scene(
    world: &mut legion::World,
    loaded: &LoadedScene,
    asset_mgr: &AssetManager,
    root_transform: TransformComponent,
) {
    // Nodi necessari alla gerarchia delle mesh
    let mut mesh_node_indices = Vec::new();

    collect_mesh_nodes(
        &loaded.nodes,
        0, // root virtuale
        &mut mesh_node_indices,
    );

    // Mantiene il mapping nodo gltf -> entity ECS
    let mut node_to_entity = vec![None; loaded.nodes.len()];

    // 1️⃣ crea solo le entity necessarie
    for node_idx in mesh_node_indices.iter() {
        let node = &loaded.nodes[*node_idx];

        // Transform di Default
        let transform = if *node_idx == 0 {
            root_transform.clone()
        } else {
            node.local_transform.clone()
        };

        let entity = world.push((
            TagComponent {
                name: node.name.clone(),
            },
            transform,
            HierarchyComponent::default(),
            GlobalModelComponent::default(),
        ));

        // Solo il nodo root identifica l'istanza della scena
        if *node_idx == 0 {
            if let Some(mut entry) = world.entry(entity) {
                entry.add_component(SceneComponent {
                    path: loaded.name.clone(),
                });
            }
        }

        node_to_entity[*node_idx] = Some(entity);
    }

    // 2️⃣ assegna mesh + material
    for node_idx in mesh_node_indices.iter() {
        let node = &loaded.nodes[*node_idx];

        if let Some(mesh_idx) = node.mesh {
            let entity = node_to_entity[*node_idx].unwrap();

            let mesh_id = &loaded.meshes[mesh_idx];

            if let Some(mut entry) = world.entry(entity) {
                entry.add_component(MeshComponent {
                    handle: mesh_id.clone(),
                });

                if let Some(mesh_asset) = asset_mgr.get::<MeshAsset>(*mesh_id) {
                    let bbox = &mesh_asset.desc.bounds;

                    entry.add_component(BoundingBoxComponent {
                        bounding_box: bbox.clone(),
                        global_bounding_box: bbox.clone(),
                    });
                }
            }
        }
    }

    // 3️⃣ collega solo la gerarchia filtrata
    for node_idx in mesh_node_indices.iter() {
        let node = &loaded.nodes[*node_idx];

        let Some(parent) = node_to_entity[*node_idx] else {
            continue;
        };

        for &child_idx in &node.children {
            let Some(child) = node_to_entity[child_idx] else {
                continue;
            };

            if let Ok(mut entry) = world.entry_mut(parent) {
                if let Ok(h) = entry.get_component_mut::<HierarchyComponent>() {
                    h.children.push(child);
                }
            }

            if let Ok(mut entry) = world.entry_mut(child) {
                if let Ok(h) = entry.get_component_mut::<HierarchyComponent>() {
                    h.parent = Some(parent);
                }
            }
        }
    }
}

// Snapshot for UI
impl Scene {
    pub fn get_selected_componet_state(
        &self,
        selected: Option<Entity>,
        asset_mgr: &AssetManager,
    ) -> UiComponentState {
        use crate::assets::{
            MaterialId, material_asset::MaterialAsset, material_desc::MaterialDesc,
        };
        use std::collections::HashMap;

        let world = &self.world;

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
                let materials: HashMap<MaterialId, MaterialDesc> = mesh_asset
                    .desc
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

impl Scene {
    pub fn get_root_snapshot(&self) -> crate::ui::RootSnapshot {
        crate::ui::RootSnapshot {
            root_nodes: get_hierarchy_roots(&self.world),
            lights_nodes: get_lights_nodes(&self.world),
        }
    }
}

/// Costruisce la gerarchia degli oggetti
fn get_hierarchy_roots(world: &legion::World) -> crate::ui::RootNodes {
    let mut roots = crate::ui::RootNodes::default();
    let mut query = <(Entity, &HierarchyComponent)>::query();

    for (entity, hierarchy) in query.iter(world) {
        if hierarchy.parent.is_none() {
            roots.nodes.push(build_node(world, *entity, None));
        }
    }

    roots
}

/// Costruisce i nodi root per le luci
fn get_lights_nodes(world: &legion::World) -> crate::ui::LightNodes {
    let mut roots = crate::ui::LightNodes::default();
    let mut query = <(Entity, &LightComponent, &TagComponent)>::query();

    for (entity, light, tag) in query.iter(world) {
        roots.nodes.push(crate::ui::LightNode {
            name: tag.name.clone(),
            comp: light.clone(), 
            entity: *entity,
        });
    }

    roots
}

/// Funzione ricorsiva che costruisce un nodo con tutti i figli
fn build_node(world: &legion::World, entity: Entity, parent: Option<Entity>) -> crate::ui::HierarchyNode {
    let entry = match world.entry_ref(entity) {
        Ok(e) => e,
        Err(_) => {
            return crate::ui::HierarchyNode {
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

    crate::ui::HierarchyNode {
        name,
        parent,
        entity,
        children,
    }
}

