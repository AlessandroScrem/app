use super::*;
use crate::app::domain::events::{AssetEvent, DomainEvent, EntityEvent, SelectionEvent};
use imgui::*;
use legion::Entity;

pub struct EntityListUi {}

impl Layer for EntityListUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        ui.window("Entities")
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(|| {
                draw_hierarchy_nodes(ui, ctx);
                draw_lights_nodes(ui, ctx);

                if ui.is_window_hovered()
                    && !ui.is_any_item_hovered()
                    && ui.is_mouse_clicked(imgui::MouseButton::Right)
                {
                    ui.open_popup("context");
                }

                if let Some(popup) = ui.begin_popup("context") {
                    ui.menu_item("Load Gltf ..").then(|| {
                        menu_bar::file_open(FileFilter::Gltf)
                            .map(|f| ctx.write.push(DomainEvent::Assets(AssetEvent::LoadGltf(f))));
                    });
                    ui.menu_item("Add Light..").then(|| {
                        ctx.write.push(DomainEvent::Entity(EntityEvent::AddLight));
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
    let children = &node.children;
    let is_active = selected.is_some_and(|e| e == entity);

    let flags = children
        .is_empty()
        .then_some(TreeNodeFlags::LEAF)
        .unwrap_or(TreeNodeFlags::empty())
        | is_active
            .then_some(TreeNodeFlags::SELECTED)
            .unwrap_or(TreeNodeFlags::empty());

    let icon = if node.parent.is_none() {
        ICON_LAYER_DOT
    } else if is_active {
        ICON_LAYER_ACTIVE
    } else {
        ICON_LAYER
    };

    let label = format!("{icon} {}", node.name,); // ◈ Name
    let opened = ui
        .tree_node_config(label)
        .flags(flags)
        .default_open(true)
        .push();

    let clicked = ui.is_item_clicked();

    if let Some(_token) = opened {
        for child in children {
            draw_entity_node_recurse(ui, child, selected);
        }
    }

    if clicked {
        *selected = Some(entity);  
    };

}

fn draw_hierarchy_nodes(ui: &imgui::Ui, ctx: &mut UiContext) {
    ui.text("Meshes");
    ui.separator();
    let mut selected = ctx.snapshot.selected.clone();

    ui.group(|| {
        for node in ctx.snapshot.root_snapshot.root_nodes.nodes.iter() {
            // traverse from root nodes
            draw_entity_node_recurse(ui, node, &mut selected);
        }
    });

    // Commands on selected
    if let Some(selected) = selected.clone() {
        if let Some(node) = ctx
            .snapshot
            .root_snapshot
            .root_nodes
            .nodes
            .iter()
            .find(|n| n.entity == selected)
        {
            if node.parent.is_none() {
                // add Context menu if ui.group is hovered
                if ui.is_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                    ui.open_popup("entity_context");
                }
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

const ICON_LIGHTBULB: &str = "\u{EA61}"; // 💡
const ICON_TRASH: &str = "\u{EA81}"; // 🗑
const ICON_ADD: &str = "\u{EA60}"; //➕
const ICON_GEAR: &str = "\u{EAF8}"; //⚙
const ICON_EYE: &str = "\u{EA70}"; // 👁

const ICON_LAYER: &str = "\u{EBD2}"; // ◈
const ICON_LAYER_DOT: &str = "\u{EBD3}"; // ◈
const ICON_LAYER_ACTIVE: &str = "\u{EBD4}"; // ◈

fn draw_lights_nodes(ui: &imgui::Ui, ctx: &mut UiContext) {
    let selected = ctx.snapshot.selected;
    let lights_nodes = &ctx.snapshot.root_snapshot.lights_nodes;

    ui.separator();
    ui.text("Lights");

    let style = ui.clone_style();

    let padding = style.frame_padding[0];
    let buttons = 4.0;
    let spacing = style.item_spacing[0] * buttons - 1.0;

    let button_width = ui.calc_text_size(ICON_TRASH)[0] + padding * 2.0;
    let total_width = buttons * button_width + spacing;

    for (i, node) in lights_nodes.nodes.iter().enumerate() {
        let entity = node.entity;
        let label = format!("{ICON_LIGHTBULB} {}:{:?}", node.name, i); // 💡 name:#

        let flags = TreeNodeFlags::LEAF
            | if selected == Some(entity) {
                TreeNodeFlags::SELECTED
            } else {
                TreeNodeFlags::empty()
            };

        let opened = ui.tree_node_config(label.clone()).flags(flags).push();
        let clicked = ui.is_item_clicked();

        // Right Buttons           👁 + 🗑 ⚙
        // Spinge i pulsanti a destra
        ui.same_line();
        ui.same_line_with_pos(ui.window_content_region_max()[0] - total_width);

        if ui.small_button(ICON_EYE) {
            // visible
            let mut light = node.comp.clone();
            light.enabled = !light.enabled;
            ctx.write.push(DomainEvent::Entity(EntityEvent::UpdateLight(
                node.entity.clone(),
                light,
            )));
        }
        ui.same_line();
        if ui.small_button(ICON_ADD) {
            // Add
            ctx.write.push(DomainEvent::Entity(EntityEvent::AddLight));
        }

        ui.same_line();
        if ui.small_button(ICON_TRASH) {
            // Delete
            ctx.write
                .push(DomainEvent::Entity(EntityEvent::RemoveEntity(entity)));
        }

        ui.same_line();
        if ui.small_button(ICON_GEAR) {
            // property
            ctx.write
                .push(DomainEvent::Selection(SelectionEvent::Selected(Some(
                    entity,
                ))));
        }
        if let Some(_token) = opened {}

        if clicked {
            ctx.write
                .push(DomainEvent::Selection(SelectionEvent::Selected(Some(
                    entity,
                ))));
        }
    }
}
