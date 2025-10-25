use super::*;
use legion::World;

pub fn draw_window_properties(world: &mut World, ctx: &mut InspectorContext) {
    ctx.ui
        .window("Properties")
        .size([300.0, 300.0], Condition::FirstUseEver)
        .build(|| {
            inspector::draw_entity_inspector(world, ctx);
        });
}
