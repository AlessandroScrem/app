use legion::*;

pub struct Scene {
    pub world: World,
    pub schedule: Schedule,
}

impl Default for Scene {
    fn default() -> Self {
        let world = World::default();

        let mut schedule_builder = Schedule::builder();
        let schedule = schedule_builder.build();

        Scene { world, schedule }
    }
    
}
