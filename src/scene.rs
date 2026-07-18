use std::{collections::VecDeque, fs, path::Path};

use legion::*;
use serde::{Deserialize, Serialize};

use crate::{
    app::domain::events::{DomainEvent, SceneEvent},
    assets::{
        MeshAsset,
        asset_manager::AssetManager,
        gltf_loader::{LoadedScene, NodeData, load_gltf},
    },
    ecs::components::*,
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
    pub schedule: Schedule,
}

impl Default for Scene {
    fn default() -> Self {
        let world = World::default();

        let mut schedule_builder = Schedule::builder();
        let schedule = schedule_builder.build();

        Scene {
            world,
            schedule,
            filename: None,
        }
    }
}

impl Scene {
    pub fn save(&mut self, event_queue: &mut VecDeque<DomainEvent>) {
        if let Some(filename) = self.filename.as_ref() {
            event_queue.push_back(DomainEvent::Scene(SceneEvent::SaveAs(filename.into())));
        } else {
        }
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

pub fn save_scene_json<P: AsRef<Path>>(
    world: &legion::World,
    filename: P,
) -> anyhow::Result<String, anyhow::Error> {
    let mut scenes = Vec::new();

    let mut query = <(&SceneComponent, &TransformComponent)>::query();

    for (scene, transform) in query.iter(world) {
        scenes.push(SceneEntry {
            path: scene.path.clone(),
            transform: transform.clone(),
        });
    }

    let file = SceneFile { version: 1, scenes };

    let json = serde_json::to_string_pretty(&file)?;

    fs::write(&filename, json)?;
    let string_name = filename.as_ref().to_string_lossy().to_string();

    Ok(string_name)
}

pub fn open_scene<P: AsRef<Path>>(
    filename: P,
    asset_mgr: &mut AssetManager,
    event_queue: &mut VecDeque<DomainEvent>,
) -> anyhow::Result<String, anyhow::Error> {
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
            event_queue.push_back(DomainEvent::Scene(SceneEvent::AddComponent(
                loaded,
                root_transform,
            )));
        }
    }

    let string_name = filename.as_ref().to_string_lossy().to_string();

    Ok(string_name)
}
