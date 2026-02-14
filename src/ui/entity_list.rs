use super::*;
use imgui::*;
use legion::Entity;

pub struct EntityListUi {}

impl Layer for EntityListUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        ui.window("Entities")
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(|| {
                draw_hierarchy_nodes(ui, ctx);
                ui.separator();
                draw_lights_nodes(ui, ctx);

                if ui.is_window_hovered()
                    && !ui.is_any_item_hovered()
                    && ui.is_mouse_clicked(imgui::MouseButton::Right)
                {
                    ui.open_popup("context");
                }

                if let Some(popup) = ui.begin_popup("context") {
                    ui.menu_item("Load Gltf ..").then(|| {
                        rfd::FileDialog::new()
                            .add_filter("gltf", &["gltf"])
                            .pick_file()
                            .map(|f| ctx.write.push(DomainEvent::Assets(AssetEvent::LoadGltf(f))));
                    });
                    popup.end();
                }

                // deselect if clicked on empty
                if ui.is_window_hovered()
                    && ui.is_mouse_clicked(MouseButton::Left)
                    && !ui.is_any_item_hovered()
                {
                    ctx.write
                        .push(DomainEvent::Selection(SelectionEvent::Selected(None)));
                }
            });
    }
}

fn draw_entity_node_recurse(ui: &Ui, node: &HierarchyNode, selected: &mut Option<Entity>) {
    let entity = node.entity;
    let name = &node.name;
    let children = &node.children;

    let mut flags = TreeNodeFlags::SPAN_AVAIL_WIDTH;
    if children.is_empty() {
        flags |= TreeNodeFlags::LEAF;
    };
    if selected.is_some_and(|e| e == entity) {
        flags |= TreeNodeFlags::SELECTED;
    }

    let label = if name.is_empty() { "##Node" } else { name };
    ui.tree_node_config(label)
        .flags(flags)
        .default_open(true)
        .build(|| {
            // Controlla se il nodo è stato cliccato e aggiorna la selezione
            if ui.is_item_clicked() {
                *selected = Some(entity);
            }
            for child in children {
                draw_entity_node_recurse(ui, child, selected);
            }
        });
}

fn draw_hierarchy_nodes(ui: &imgui::Ui, ctx: &mut UiContext) {
    let mut selected = ctx.snapshot.selected.clone();

    ui.group(|| {
        for node in ctx.snapshot.root_snapshot.root_nodes.nodes.iter() {
            // traverse from root nodes
            draw_entity_node_recurse(ui, node, &mut selected);
        }
    });

    // Commands on selected
    if let Some(selected) = selected.clone() {
        if ctx
            .snapshot
            .root_snapshot
            .root_nodes
            .nodes
            .iter()
            .any(|n| n.parent == None)
        {
            // add Context menu if ui.group is hovered
            if ui.is_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                ui.open_popup("entity_context");
            }
            if let Some(popup) = ui.begin_popup("entity_context") {
                ui.menu_item("Remove ..").then(|| {
                    ctx.write
                        .push(DomainEvent::Entity(EntityEvent::RemoveEntity(selected)));
                });
                ui.menu_item("Add Parent ..").then(|| {
                    ctx.write
                        .push(DomainEvent::Entity(EntityEvent::AddParent(selected)))
                });
                popup.end();
            }
        }
    }

    if selected != ctx.snapshot.selected.clone() {
        ctx.write
            .push(DomainEvent::Selection(SelectionEvent::Selected(selected)));
    }
}

fn draw_lights_nodes(ui: &imgui::Ui, ctx: &mut UiContext) {
    let selected = ctx.snapshot.selected;
    let lights_nodes = &ctx.snapshot.root_snapshot.lights_nodes;

    for node in lights_nodes.nodes.iter() {
        let entity = node.entity;
        if ui
            .selectable_config(format!("{} {:?}", node.name, node.entity))
            .selected(selected.map(|e| e == entity).unwrap_or(false))
            .build()
        {
            ctx.write
                .push(DomainEvent::Selection(SelectionEvent::Selected(Some(
                    entity,
                ))));
        }
    }
}
