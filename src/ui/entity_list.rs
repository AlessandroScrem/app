use super::*;
use crate::app::domain::events::{
    AssetEvent::{LoadGltf},
    DomainEvent::{self, Assets, Selection},
    EntityEvent::{AddLight, AddParent, RemoveEntity, UpdateLight},
    SelectionEvent::{Select},
};
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
                        menu_bar::file_open(FileFilter::Gltf).map(|f| {
                            ctx.bus
                                .send_domain(DomainEvent::Assets(LoadGltf(f)))
                        });
                    });
                    ui.menu_item("Add Light..").then(|| {
                        ctx.bus
                            .send_domain(DomainEvent::Entity(AddLight));
                    });
                    popup.end();
                }

                // deselect if clicked on empty
                if ui.is_window_hovered()
                    && ui.is_mouse_clicked(MouseButton::Left)
                    && !ui.is_any_item_hovered()
                {
                    ctx.bus
                        .send_domain(Selection(Select(None)));
                }
            });
    }
}

fn draw_entity_node_recurse(
    ui: &Ui,
    node: &HierarchyNode,
    selected: &mut Option<Entity>,
    ctx: &mut UiContext,
) {
    let entity = node.entity;
    let children = &node.children;
    let is_active = selected.is_some_and(|e| e == entity);
    let is_root = node.parent.is_none();

    let flags = children
        .is_empty()
        .then_some(TreeNodeFlags::LEAF)
        .unwrap_or(TreeNodeFlags::empty())
        | is_active
            .then_some(TreeNodeFlags::SELECTED)
            .unwrap_or(TreeNodeFlags::empty());

    let icon = if is_root {
        ICON_LAYER_DOT
    } else if is_active {
        ICON_LAYER_ACTIVE
    } else {
        ICON_LAYER
    };

    const BUTTONS: usize = 4;

    let label = format!("{icon} {}", node.name,); // ◈ Name
    let opened = ui
        .tree_node_config(label)
        .flags(flags)
        .default_open(true)
        .push();

    if is_root {
        // Right Buttons           👁 + 🗑 ⚙
        right_buttons(ui, BUTTONS, |ui| {
            if ui.small_button(ICON_EYE) {
                // Enabled
                // node.enabled = !node.enabled;
            }
            ui.same_line();
            if ui.small_button(ICON_ADD) {
                // Add
                menu_bar::file_open(FileFilter::Gltf)
                    .map(|f| ctx.bus.send_domain(Assets(LoadGltf(f))));
            }

            ui.same_line();
            if ui.small_button(ICON_TRASH) {
                // Delete
                ctx.bus.send_domain(DomainEvent::Entity(RemoveEntity(entity)));
            }

            ui.same_line();
            if ui.small_button(ICON_GEAR) {
                // property
                ctx.bus.send_domain(Selection(Select(Some(entity))));
            }
        });
    }

    let clicked = ui.is_item_clicked();

    if let Some(_token) = opened {
        for child in children {
            draw_entity_node_recurse(ui, child, selected, ctx);
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
            draw_entity_node_recurse(ui, node, &mut selected, ctx);
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
                    ctx.bus.send_domain(DomainEvent::Entity(RemoveEntity(selected)));
                });
                ui.menu_item("Add Parent ..")
                    .then(|| ctx.bus.send_domain(DomainEvent::Entity(AddParent(selected))));
                popup.end();
            }
        }
    }

    if selected != ctx.snapshot.selected.clone() {
        ctx.bus.send_domain(Selection(Select(selected)));
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

    const BUTTONS: usize = 4;

    for (i, node) in lights_nodes.nodes.iter().enumerate() {
        let entity = node.entity;
        let mut light = node.comp.clone();
        let label = format!("{ICON_LIGHTBULB} {}:{:?}", node.name, i); // 💡 name:#

        let flags = TreeNodeFlags::LEAF
            | if selected == Some(entity) {
                TreeNodeFlags::SELECTED
            } else {
                TreeNodeFlags::empty()
            };

        let _disabled = disabled_style(ui, light.enabled);

        let opened = ui.tree_node_config(label.clone()).flags(flags).push();
        let clicked = ui.is_item_clicked();

        // Right Buttons           👁 + 🗑 ⚙
        right_buttons(ui, BUTTONS, |ui| {
            if ui.small_button(ICON_EYE) {
                // Enabled
                light.enabled = !light.enabled;
                ctx.bus
                    .send_domain(DomainEvent::Entity(UpdateLight(node.entity.clone(), light)));
            }
            ui.same_line();
            if ui.small_button(ICON_ADD) {
                // Add
                ctx.bus.send_domain(DomainEvent::Entity(AddLight));
            }

            ui.same_line();
            if ui.small_button(ICON_TRASH) {
                // Delete
                ctx.bus.send_domain(DomainEvent::Entity(RemoveEntity(entity)));
            }

            ui.same_line();
            if ui.small_button(ICON_GEAR) {
                // property
                ctx.bus.send_domain(Selection(Select(Some(entity))));
            }
        });

        if let Some(_token) = opened {}

        if clicked {
            // Select
            ctx.bus.send_domain(Selection(Select(Some(entity))));
        }
    }
}

fn disabled_style(ui: &Ui, enabled: bool) -> Option<imgui::ColorStackToken<'_>> {
    (!enabled).then(|| ui.push_style_color(StyleColor::Text, [1.0, 1.0, 1.0, 0.35]))
}

fn right_buttons<F>(ui: &Ui, buttons: usize, f: F)
where
    F: FnOnce(&Ui),
{
    let style = ui.clone_style();

    let padding = style.frame_padding[0];
    let spacing = style.item_spacing[0] * buttons as f32 - 1.0;

    let button_width = ui.calc_text_size(ICON_TRASH)[0] + padding * 2.0;
    let total_width = buttons as f32 * button_width + spacing;

    ui.same_line();
    // Push buttons to right
    ui.same_line_with_pos(ui.window_content_region_max()[0] - total_width);

    f(ui);
}
