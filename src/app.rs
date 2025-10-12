use std::sync::Arc;
use std::time::Instant;
use crate::input::Input;

use crate::Globals;
use crate::prelude::imgui_tools::ImguiState;
use crate::prelude::*;
use crate::scene::Scene;

use legion::systems::Builder;
use legion::Resources;
use legion::Schedule;

pub struct App {
    pub(super) window: Option<Arc<winit::window::Window>>,
    pub current_scene: Scene,
    pub resources: Resources,

    pub size: winit::dpi::PhysicalSize<u32>,
    pub clock: Instant,
    pub fixed_timestep: f32,
    pub elapsed_time: f32,
    pub frame_time: f32,
    pub delta_time: f32,
    pub update_schedule: Schedule,
    pub render_schedule: Schedule,
    pub imgui: Option<ImguiState>,
    pub is_minimized: bool,
}

impl Default for App {
    fn default() -> Self {
        let update_schedule = Builder::default().build();
        let render_schedule = Builder::default().build();

        Self {
            window: None,
            current_scene: Scene::default(),
            resources: Resources::default(),
            update_schedule,
            render_schedule,
            
            size:  winit::dpi::PhysicalSize::new(1280, 1024),
            clock: Instant::now(),
            fixed_timestep: 1.0 / 60.0,
            elapsed_time: 0.0,
            frame_time: 0.0,
            delta_time: 0.0,
            imgui: None,
            is_minimized: false,
        }
    }
}

impl App {
    pub fn new_with_size(width: u32, height: u32) -> Self {
        Self {
            size: winit::dpi::PhysicalSize::new(width, height),
            ..Default::default()
        }
    }

    pub fn load(&mut self) {
        let timer = std::time::Instant::now();

        self.resources.insert(Input::new());
        self.resources.insert(Camera::default());
        self.resources.insert(Globals::default());

        crate::entities::mesh::create(&mut self.current_scene.world, &self.resources);
        crate::entities::light::create(&mut self.current_scene.world, &self.resources);

        self.current_scene.schedule = crate::systems::create_current_scene_schedule_builder();
        self.update_schedule = crate::systems::create_update_schedule_builder();
        self.render_schedule = crate::systems::create_render_schedule_builder();

        self.create_gui();

        info!("App loader took {} ms", timer.elapsed().as_millis());
    }

    fn create_gui(&mut self) {
        if let Some(window) = &self.window {
            let imgui = imgui_tools::ImguiState::create_imgui(window, &mut self.resources);

            self.imgui = Some(imgui);
        }
    }
}

