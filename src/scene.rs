use crate::input::Input;
use legion::*;

use super::DeltaTime;

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

    pub(crate) fn update(&mut self, delta_time: f32, resources: &mut Resources) {
        {
            let mut delta = resources.get_mut::<DeltaTime>().unwrap();
            *delta = DeltaTime(delta_time);
        }

        self.schedule.execute(&mut self.world, resources);
    }
}
