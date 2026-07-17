use legion::*;

use crate::{
    assets::{MeshAsset, asset_manager::AssetManager, gltf_loader::{LoadedScene, NodeData}}, ecs::components::*,
};

pub struct Scene {
    pub world: World,
    pub schedule: Schedule,
}

impl Default for Scene {
    fn default() -> Self {
        let world = World::default();

        let mut schedule_builder = Schedule::builder();
        let schedule = schedule_builder.build();

        Scene { world, schedule }
    }
}

fn collect_mesh_nodes(
    nodes: &[NodeData],
    index: usize,
    output: &mut Vec<usize>,
) -> bool {
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

pub fn spawn_scene_transform(
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

                    entry.add_component(
                        BoundingBoxComponent {
                            bounding_box: bbox.clone(),
                            global_bounding_box: bbox.clone(),
                        }
                    );
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
