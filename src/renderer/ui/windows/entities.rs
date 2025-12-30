use super::*;
use legion::query::IntoQuery;
use legion::{Entity, EntityStore, World, component};
use crate::prelude::ui::state::UiEvent;
use crate::{HierarchyComponent, TagComponent, picking::PickObject};

pub fn draw_window_entities(world: &mut World, ctx: &mut InspectorContext) {
    let ui = ctx.ui;

    ui.window("Entities")
        .size([300.0, 100.0], Condition::FirstUseEver)
        .build(|| {
            draw_hierarchy_nodes(world, ctx);
            ui.separator();
            draw_without_hierarchy(world, ctx);

            if ui.is_window_hovered()
                && !ui.is_any_item_hovered()
                && ui.is_mouse_clicked(imgui::MouseButton::Right)
            {
                ui.open_popup("context");
            }

            if let Some(popup) = ui.begin_popup("context") {
                ctx.ui.menu_item("Load Gltf ..").then(|| {
                    rfd::FileDialog::new()
                        .add_filter("gltf", &["gltf"])
                        .pick_file()
                        .map(|f| ctx.command = Some(UiEvent::LoadGltf(f)));
                });
                popup.end();
            }

            // deselect if clicked on empty
            handle_deselection(ctx);
        });
}

fn handle_deselection(ctx: &mut InspectorContext) {
    let mut pick_object = ctx.resources.get_mut::<PickObject>().unwrap();
    let ui = ctx.ui;
    // Deseleziona solo se clicchi nella finestra stessa
    // ma non sopra un widget/interazione
    if ui.is_window_hovered() && ui.is_mouse_clicked(MouseButton::Left) && !ui.is_any_item_hovered()
    {
        pick_object.select(None);
    }
}

fn is_root_node(entity: Entity, world: &mut World) -> bool {
    world.entry(entity).is_some_and(|e| {
        e.get_component::<HierarchyComponent>()
            .is_ok_and(|h| h.parent.is_none())
    })
}

fn draw_entity_node_recurse(ui: &Ui, entity: Entity, world: &World, pick_object: &mut PickObject) {
    let (name, children) = {
        let entry = world.entry_ref(entity).unwrap();
        let tag = entry.get_component::<TagComponent>().unwrap();
        let hierarchy = entry.get_component::<HierarchyComponent>().unwrap();
        (tag.name.clone(), hierarchy.children.clone())
    };

    let is_selected = pick_object.selected.is_some_and(|e| e == entity);
    let flags = TreeNodeFlags::SPAN_AVAIL_WIDTH;
    let flags = if children.is_empty() {
        flags | TreeNodeFlags::LEAF
    } else {
        flags
    };
    let flags = if is_selected {
        flags | TreeNodeFlags::SELECTED
    } else {
        flags
    };

    ui.tree_node_config(name.clone())
        .flags(flags)
        .default_open(true)
        .build(|| {
            // Controlla se il nodo è stato cliccato e aggiorna la selezione
            if ui.is_item_clicked() {
                pick_object.select(Some(entity));
            }
            for child in children {
                draw_entity_node_recurse(ui, child, world, pick_object);
            }
        });
}

fn draw_hierarchy_nodes(world: &mut World, ctx: &mut InspectorContext) {
    let mut pick_object = ctx.resources.get_mut::<PickObject>().unwrap();
    let mut hierarchy_query = <(Entity, &HierarchyComponent)>::query();
    let ui = ctx.ui;

    ui.group(|| {
        for (entity, hirarchy) in hierarchy_query.iter(world) {
            // traverse from root nodes
            if hirarchy.parent.is_none() {
                draw_entity_node_recurse(ui, entity.clone(), world, &mut pick_object);
            }
        }
    });

    // Add Parent to node
    if let Some(selected) = pick_object.selected {
        if is_root_node(selected, world) {
            // add Context menu if ui.group is hovered
            if ui.is_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                ui.open_popup("entity_context");
            }
            if let Some(popup) = ui.begin_popup("entity_context") {
                ui.menu_item("Remove ..").then(|| {
                    ctx.command = Some(UiEvent::RemoveEntity(selected));
                    pick_object.selected = None;
                    ctx.selected = None;
                });
                ui.menu_item("Add Parent ..")
                    .then(|| ctx.command = Some(UiEvent::AddParent(selected)));
                popup.end();
            }
        }
    }
}

fn draw_without_hierarchy(world: &mut World, ctx: &mut InspectorContext) {
    let ui = ctx.ui;
    let mut no_hierarchy_query =
        <(Entity, &TagComponent)>::query().filter(!component::<HierarchyComponent>());

    let mut pick_object = ctx.resources.get_mut::<PickObject>().unwrap();

    for (entity, tag) in no_hierarchy_query.iter(world) {
        if ui
            .selectable_config(format!("{} {:?}", tag.name, entity))
            .selected(pick_object.selected.map(|e| e == *entity).unwrap_or(false))
            .build()
        {
            pick_object.select(Some(*entity));
        }
    }
}
