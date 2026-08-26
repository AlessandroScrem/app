use super::*;
use crate::editor::{EditorCommand, EntityId, HierarchyData, HierarchyNode};
use crate::ui::ui_layer::Layer;
use imgui::*;
use std::collections::HashSet;

const ICON_LIGHTBULB: &str = "\u{EA61}";
const ICON_TRASH: &str = "\u{EA81}";
const ICON_ADD: &str = "\u{EA60}";
const ICON_FOLDER: &str = "\u{EAF7}";
const ICON_CLEAR: &str = "\u{EAC0}";
const ICON_EYE: &str = "\u{EA70}";
const ICON_LAYER: &str = "\u{EBD2}";
const ICON_LAYER_DOT: &str = "\u{EBD3}";
const ICON_LAYER_ACTIVE: &str = "\u{EBD4}";

pub struct EntityListUi;
impl Layer for EntityListUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        ui.window("Entities").size([340.0, 420.0], Condition::FirstUseEver).build(|| {
            // Keep the frequently used editor actions available without forcing the user
            // through the main menu. They still go through the editor protocol.
            if ui.small_button(format!("{ICON_FOLDER}##load_gltf")) {
                ui.tooltip_text("Load glTF / GLB");
                if let Some(path) = crate::ui::menu_bar::file_open(crate::ui::menu_bar::FileFilter::Gltf) {
                    ctx.connection.commands.send(EditorCommand::LoadGltf { path });
                }
            }
            if ui.is_item_hovered() { ui.tooltip_text("Load glTF / GLB"); }
            ui.same_line();
            if ui.small_button(format!("{ICON_LIGHTBULB}##add_light")) {
                ctx.connection.commands.send(EditorCommand::AddLight);
            }
            if ui.is_item_hovered() { ui.tooltip_text("Add light"); }
            ui.same_line();
            if ui.small_button(format!("{ICON_CLEAR}##clear_selection")) {
                ctx.connection.commands.send(EditorCommand::Select { entities: Vec::new() });
            }
            if ui.is_item_hovered() { ui.tooltip_text("Clear selection"); }
            ui.separator();

            let Some(hierarchy) = ctx.hierarchy else { ui.text("Loading hierarchy..."); return; };
            let selection: HashSet<EntityId> = ctx.selection.iter().copied().collect();
            let mut clicked = None;
            let mut action = None;
            for node in hierarchy.nodes.iter().filter(|node| node.parent.is_none()) {
                draw_node(ui, node, hierarchy, &selection, &mut clicked, &mut action);
            }
            if let Some(entity) = clicked {
                ctx.connection.commands.send(EditorCommand::Select { entities: vec![entity] });
            }
            if let Some(command) = action { ctx.connection.commands.send(command); }
            if ui.is_window_hovered() && !ui.is_any_item_hovered() && ui.is_mouse_clicked(MouseButton::Left) {
                ctx.connection.commands.send(EditorCommand::Select { entities: Vec::new() });
            }
            if let Some(popup) = ui.begin_popup_context_window() {
                if ui.menu_item("Load Gltf ..") {
                    if let Some(path) = crate::ui::menu_bar::file_open(crate::ui::menu_bar::FileFilter::Gltf) {
                        ctx.connection.commands.send(EditorCommand::LoadGltf { path });
                    }
                }
                if ui.menu_item("Add Light") { ctx.connection.commands.send(EditorCommand::AddLight); }
                popup.end();
            }
        });
    }
}

fn draw_node(ui: &Ui, node: &HierarchyNode, hierarchy: &HierarchyData, selection: &HashSet<EntityId>, clicked: &mut Option<EntityId>, action: &mut Option<EditorCommand>) {
    let is_selected = selection.contains(&node.entity);
    let children: Vec<&HierarchyNode> = hierarchy.nodes.iter().filter(|child| child.parent == Some(node.entity)).collect();
    let flags = (if children.is_empty() { TreeNodeFlags::LEAF } else { TreeNodeFlags::empty() }) | if is_selected { TreeNodeFlags::SELECTED } else { TreeNodeFlags::empty() };
    let icon = if node.is_light { ICON_LIGHTBULB } else if node.parent.is_none() { ICON_LAYER_DOT } else if is_selected { ICON_LAYER_ACTIVE } else { ICON_LAYER };
    let _disabled = (!node.visible).then(|| ui.push_style_color(StyleColor::Text, [1.0, 1.0, 1.0, 0.35]));
    let opened = ui.tree_node_config(format!("{icon} {}##{}", node.name, node.entity)).flags(flags).default_open(true).push();
    if ui.is_item_clicked() { *clicked = Some(node.entity); }
    right_icons(ui, |ui| {
        if ui.small_button(format!("{ICON_EYE}##{}", node.entity)) { *action = Some(EditorCommand::SetEntityEnabled { entity: node.entity, enabled: !node.visible }); }
        if ui.is_item_hovered() { ui.tooltip_text(if node.visible { "Hide" } else { "Show" }); }
        ui.same_line();
        if ui.small_button(format!("{ICON_ADD}##{}", node.entity)) {
            if let Some(path) = crate::ui::menu_bar::file_open(crate::ui::menu_bar::FileFilter::Gltf) { *action = Some(EditorCommand::LoadGltf { path }); }
        }
        if ui.is_item_hovered() { ui.tooltip_text("Add glTF / GLB"); }
        ui.same_line();
        if ui.small_button(format!("{ICON_TRASH}##{}", node.entity)) { *action = Some(EditorCommand::Delete { entities: vec![node.entity] }); }
        if ui.is_item_hovered() { ui.tooltip_text("Delete entity"); }
    });
    if let Some(_token) = opened { for child in children { draw_node(ui, child, hierarchy, selection, clicked, action); } }
    if is_selected && ui.is_item_hovered() && ui.is_mouse_clicked(MouseButton::Right) { ui.open_popup(format!("entity_context##{}", node.entity)); }
    if let Some(popup) = ui.begin_popup(format!("entity_context##{}", node.entity)) {
        if ui.menu_item("Remove") { *action = Some(EditorCommand::Delete { entities: vec![node.entity] }); }
        if ui.menu_item("Add Parent") { *action = Some(EditorCommand::AddParent { entity: node.entity }); }
        popup.end();
    }
}

fn right_icons<F: FnOnce(&Ui)>(ui: &Ui, f: F) {
    let style = ui.clone_style();
    let padding = style.frame_padding[0];
    let width = ui.calc_text_size(ICON_TRASH)[0] + padding * 2.0;
    let total = width * 3.0 + style.item_spacing[0] * 2.0;
    ui.same_line_with_pos(ui.window_content_region_max()[0] - total);
    f(ui);
}
