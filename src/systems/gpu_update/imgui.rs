
use crate::{
    BoundingBoxComponent, DomainEvents, Globals, HierarchyComponent, LightComponent,
    MeshComponent, TagComponent, TransformComponent, UiComponentView,
    application_handler::WindowEventQueue,
    assets::{material_manager::MaterialManager, mesh_manager::MeshManager},
    camera::Camera,
    picking::PickObject,
    prelude::ui::{
        ImguiState,
        state::{HierarchyNode, RootNodes, Snapshot},
    },
};

use legion::{world::SubWorld, *};

#[system]
#[read_component(HierarchyComponent)]
#[read_component(TagComponent)]
#[read_component(MeshComponent)]
#[read_component(BoundingBoxComponent)]
#[read_component(LightComponent)]
pub fn imgui_update(
    world: &SubWorld,
    #[resource] imgui: &mut ImguiState,
    #[resource] event_queue: &mut WindowEventQueue,
    #[resource] camera: &mut Camera,
    #[resource] picking: &mut PickObject,
    #[resource] globals: &mut Globals,
    #[resource] comp_view: &mut UiComponentView,
    #[resource] mat_mgr: &MaterialManager,
    #[resource] mesh_mgr: &MeshManager,
    #[resource] de: &mut DomainEvents,
) {
    let window = &event_queue.window;
    let selected = &mut picking.selected;
    let hovered = picking.hovered;

    *comp_view = get_comp_view(selected.as_ref().cloned(), world, mesh_mgr, mat_mgr);
    let root_nodes = get_hierarchy_roots(world);
    let lights_nodes = get_lights_roots(world);

    let mut snaphot = Snapshot {
        camera,
        globals,
        root_nodes: &root_nodes,
        lights_nodes: &lights_nodes,
        comp_view,
        selected,
        hovered,
    };

    let mut events = imgui.update_ui(window, &mut snaphot);

    while let Some(event) = events.pop_front() {
        de.queue.push_back(event);
    }
}

fn get_lights_roots(world: &SubWorld) -> RootNodes {
    let mut roots = RootNodes::default();
    let mut query = <(Entity, &LightComponent, &TagComponent)>::query();
    for (entity, _light, tag) in query.iter(world) {
        let name = tag.name.clone();
        let node = HierarchyNode {
            name,
            parent: None,
            entity: entity.clone(),
            children: Vec::new(),
        };

        roots.nodes.push(node);
    }
    roots
}

fn get_hierarchy_roots(world: &SubWorld) -> RootNodes {
    let mut query = <(Entity, &HierarchyComponent)>::query();
    let mut roots = RootNodes::default();
    for (entity, hierarchy) in query.iter(world) {
        if hierarchy.parent.is_none() {
            let node = build_node(world, *entity, None);
            roots.nodes.push(node);
        }
    }
    roots
}

fn get_comp_view(
    selected: Option<Entity>,
    world: &SubWorld,
    mesh_mgr: &MeshManager,
    mat_mgr: &MaterialManager,
) -> UiComponentView {
    let mut comp_view = UiComponentView::default();

    if let Some(selected) = selected {
        if let Ok(entry) = world.entry_ref(selected) {
            if let Ok(light) = entry.get_component::<LightComponent>() {
                comp_view.light = Some(light.clone());
            }
        }
        if let Ok(entry) = world.entry_ref(selected) {
            if let Ok(tag) = entry.get_component::<TagComponent>() {
                comp_view.tag = Some(tag.clone());
            }
        }
        if let Ok(entry) = world.entry_ref(selected) {
            if let Ok(transform) = entry.get_component::<TransformComponent>() {
                comp_view.transform = Some(transform.clone());
            }
        }
        if let Ok(entry) = world.entry_ref(selected) {
            if let Ok(bbox) = entry.get_component::<BoundingBoxComponent>() {
                comp_view.bounding_box = Some(bbox.clone());
            }
        }
        if let Ok(entry) = world.entry_ref(selected) {
            if let Ok(mesh) = entry.get_component::<MeshComponent>() {
                let material_handle = mesh_mgr.get_material(mesh.handle);
                let material = mat_mgr.get(material_handle).material_pbr.clone();
                comp_view.mesh = Some(mesh.clone());
                comp_view.material = Some(material);
            }
        }
    }
    comp_view
}

fn build_node(world: &SubWorld, entity: Entity, parent: Option<Entity>) -> HierarchyNode {
    let entry = world.entry_ref(entity).unwrap();

    let name = entry
        .get_component::<TagComponent>()
        .map(|n| n.name.clone())
        .unwrap_or("<unnamed>".to_string());

    let hierarchy = entry.get_component::<HierarchyComponent>().unwrap();

    let children = hierarchy
        .children
        .iter()
        .map(|&child| build_node(world, child, Some(entity)))
        .collect();

    HierarchyNode {
        name,
        parent,
        entity,
        children,
    }
}

#[system]
#[write_component(TagComponent)]
#[write_component(MeshComponent)]
#[write_component(TransformComponent)]
#[write_component(LightComponent)]
pub fn imgui_flush_selected(
    world: &mut SubWorld,
    #[resource] picking: &mut PickObject,
    #[resource] comp_view: &mut UiComponentView,
    #[resource] mesh_manager: &MeshManager,
    #[resource] material_manager: &mut MaterialManager,
) {
    if let Some(entity) = picking.selected
        && comp_view.dirty
    {
        println!("Dirty");
        if let Ok(mut entry) = world.entry_mut(entity) {
            if let Ok(tag) = entry.get_component_mut::<TagComponent>() {
                comp_view.tag.as_ref().map(|t| *tag = t.clone());
            }
            if let Ok(transform) = entry.get_component_mut::<TransformComponent>() {
                comp_view.transform.as_ref().map(|t| *transform = t.clone());
            }
            if let Ok(light) = entry.get_component_mut::<LightComponent>() {
                comp_view.light.as_ref().map(|t| *light = t.clone());
            }

            if let Some(updated_material) = comp_view.material.clone() {
                if let Ok(mesh) = entry.get_component_mut::<MeshComponent>() {
                    let material_handle = mesh_manager.get_material(mesh.handle);
                    let material = &mut material_manager.get_mut(material_handle).material_pbr;
                    *material = updated_material;
                }
            }
        }
        comp_view.dirty = false
    }
}

