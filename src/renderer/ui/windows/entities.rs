use super::*;
use crate::prelude::ui::state::{HierarchyNode, UiEvent};
use legion::Entity;

pub fn draw_window_entities(ui: &imgui::Ui, ctx: &mut UiContext) {
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
                        .map(|f| ctx.command.push_back(UiEvent::LoadGltf(f)));
                });
                popup.end();
            }

            // deselect if clicked on empty
            if ui.is_window_hovered() && ui.is_mouse_clicked(MouseButton::Left) && !ui.is_any_item_hovered()
            {
                *ctx.snapshot.selected = None;
            }
        });
}


fn draw_entity_node_recurse(ui: &Ui, node: &HierarchyNode, selected: &mut Option<Entity>) {
    let entity = node.entity;
    let name = &node.name;
    let children = &node.children;


    let is_selected = selected.is_some_and(|e| e == entity);
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
                *selected = Some(entity);
            }
            for child in children {
                draw_entity_node_recurse(ui, child, selected);
            }
        });
}

fn draw_hierarchy_nodes(ui: &imgui::Ui, ctx: &mut UiContext) {
    let selected = &mut ctx.snapshot.selected;

    ui.group(|| {
        for node in ctx.snapshot.root_nodes.nodes.iter() {
            // traverse from root nodes
            draw_entity_node_recurse(ui, node, selected);
        }
    });

    // // Add Parent to node
    // if let Some(selected) = selected {
    //     if is_root_node(*selected,) {
    //         // add Context menu if ui.group is hovered
    //         if ui.is_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
    //             ui.open_popup("entity_context");
    //         }
    //         if let Some(popup) = ui.begin_popup("entity_context") {
    //             ui.menu_item("Remove ..").then(|| {
    //                 ctx.command = Some(UiEvent::RemoveEntity(*selected));
    //                 ctx.selected = None;
    //             });
    //             ui.menu_item("Add Parent ..")
    //                 .then(|| ctx.command = Some(UiEvent::AddParent(*selected)));
    //             popup.end();
    //         }
    //     }
    // }
}

fn draw_lights_nodes(ui: &imgui::Ui, ctx: &mut UiContext) {
    let selected = &mut ctx.snapshot.selected;
    
    for node in ctx.snapshot.lights_nodes.nodes.iter() {
        let entity = node.entity;
        if ui
            .selectable_config(format!("{} {:?}", node.name, node.entity))
            .selected(selected.map(|e| e == entity).unwrap_or(false))
            .build()
        {
            **selected = Some(entity);
        }
    }
} 
