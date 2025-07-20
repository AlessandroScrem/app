use std::time::Instant;

use super::DeltaTime;
use crate::input::Input;

use crate::model_reader;
use crate::prelude::*;
use crate::scene::Scene;

use legion::Resources;
use legion::Schedule;

pub struct App {
    pub(super) window: Option<std::sync::Arc<winit::window::Window>>,
    pub current_scene: Scene,
    pub resources: Resources,

    pub clock: Instant,
    pub fixed_timestep: f32,
    pub elapsed_time: f32,
    pub frame_time: f32,
    pub delta_time: f32,
    pub last_frame: Instant,
    pub render_schedule: Schedule,
}

impl Default for App {
    fn default() -> Self {

        let mut schedule_builder = Schedule::builder();
        let render_schedule = schedule_builder.build();

        Self {
            window: None,
            current_scene: Scene::default(),
            resources: Resources::default(),
            render_schedule,

            clock: Instant::now(),
            fixed_timestep: 1.0 / 60.0,
            elapsed_time: 0.0,
            frame_time: 0.0,
            delta_time: 0.0,
            last_frame: Instant::now(),
        }
    }
}

impl App {
    pub fn load(&mut self) {
        crate::entities::camera::create(&mut self.current_scene.world, Camera::default());
        self.resources.insert(Input::new());
        self.resources.insert(DeltaTime(10.0));
        
        self.current_scene.schedule = Schedule::builder()
            .add_system(crate::systems::camera_orbit::camera_orbit_system())
            .build();
        
        self.render_schedule = crate::systems::create_render_schedule_builder();

        crate::renderer::pipeline_manager::create_default_pipeline(&self.resources);
    }
}

impl App {
    fn _load_gltf(&mut self, path: &std::path::Path) {
        let meshes = match model_reader::load_gltf(path) {
            Ok(meshes) => meshes,
            Err(e) => {
                println!("Error loading glTF: {}", e);
                return;
            }
        };

        println!("Loaded {} meshes", meshes.len());
        println!(" {:?}", meshes);
    }
}
