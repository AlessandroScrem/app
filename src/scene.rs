use legion::*;

pub(crate) struct Scene {
    pub(crate) world: World,
    pub(crate) schedule: Schedule,
}

impl Default for Scene {
    fn default() -> Self {
        let world = World::default();

        let mut schedule_builder = Schedule::builder();
        let schedule = schedule_builder.build();

        Scene { world, schedule }
    }
    
}
