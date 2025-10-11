use legion::*;

pub struct Scene {
    pub world: World,
    pub schedule: Schedule,
}

impl Scene {
    pub fn default() -> Self {
        let world = World::default();

        let mut schedule_builder = Schedule::builder();
        let schedule = schedule_builder.build();

        Scene { world, schedule }
    }
    
}
