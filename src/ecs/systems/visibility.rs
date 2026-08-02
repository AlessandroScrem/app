
// use legion::{systems::CommandBuffer, world::SubWorld, *};
// use crate::ecs::components::{ HierarchyComponent, Hidden};

// #[system]
// #[read_component(Hidden)]
// #[read_component(HierarchyComponent)]
// pub fn update_hieararchy(world: &SubWorld, commands: &mut CommandBuffer) {
//     let mut query = <(Entity, Read<HierarchyComponent>, Read<Hidden>)>::query();

//     // Entities with a `Hidden` and NOT a `Parent`
//     // (roots of a hierarchy)
//     for (_entity, hirarchy, _hidden) in query.iter(world).filter(|(_e, h, _t)| h.parent.is_none())
//     {

//         // Propaga ai figli
//         for child in hirarchy.children.iter() {
//             propagate_recursive(world, *child, commands);
//         }
//     }
// }

// fn propagate_recursive(
//     world: &SubWorld,
//     entity: Entity,
//     commands: &mut CommandBuffer,
// ) {

//     commands.add_component(entity, Hidden{});

//     // Propaga ai figli
//     let children = {
//         let entry = match world.entry_ref(entity) {
//             Ok(e) => e,
//             Err(_) => return,
//         };

//         if let Ok(hierarchy) = entry.get_component::<HierarchyComponent>() {
//             hierarchy.children.clone()
//         } else {
//             return;
//         }
//     };

//     for child in children {
//         propagate_recursive(world, child, commands);
//     }
// }
