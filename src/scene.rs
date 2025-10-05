use super::DeltaTime;
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

    pub(crate) fn update(&mut self, delta_time: f32, resources: &mut Resources) {
        let mut delta = resources.get_mut::<DeltaTime>().unwrap();
        *delta = DeltaTime(delta_time);

    }
    
    pub fn picking_update(&self, resources: &mut Resources) {
        if let Some(id) = {
            let pick_buffer = resources.get_mut::<crate::renderer::gpu_renderer::PickBuffer>().unwrap();
            pick_buffer.read_id();
            pick_buffer.get_id_if_ready()
        } {
            resources.insert(crate::renderer::gpu_renderer::Hovered(id));
        }
    }
}
