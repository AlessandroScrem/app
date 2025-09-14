use std::sync::Arc;
use std::time::Instant;

use super::DeltaTime;
use crate::input::Input;

use crate::prelude::imgui_tools::ImguiState;
use crate::prelude::*;
use crate::scene::Scene;
use crate::Globals;

use legion::Resources;
use legion::Schedule;

pub struct App {
    pub(super) window: Option<Arc<winit::window::Window>>,
    pub current_scene: Scene,
    pub resources: Resources,

    pub clock: Instant,
    pub fixed_timestep: f32,
    pub elapsed_time: f32,
    pub frame_time: f32,
    pub delta_time: f32,
    pub last_frame: Instant,
    pub render_schedule: Schedule,
    pub imgui: Option<ImguiState>,
    pub is_minimized: bool,
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
            imgui: None,
            is_minimized: false,
        }
    }
}

impl App {
    pub fn load(&mut self) {
        self.resources.insert(Input::new());
        self.resources.insert(DeltaTime(10.0));
        self.resources.insert(Camera::default());
        self.resources.insert(Globals::default());
        
        crate::entities::mesh::create(&mut self.current_scene.world, &self.resources); 
        crate::entities::light::create(&mut self.current_scene.world, &self.resources);

        self.current_scene.schedule = Schedule::builder()
            .add_system(crate::systems::camera_orbit::camera_orbit_system())
            .build();

        self.render_schedule = crate::systems::create_render_schedule_builder();

        self.create_gui();

    }

    fn create_gui(&mut self) {
        if let Some(window) = &self.window {
            let imgui = imgui_tools::ImguiState::create_imgui(window, &mut self.resources);
            
            self.imgui = Some(imgui);
        }
    }
}

/* /// Create a BRDF LUT and store it in the texture manager
use crate::renderer;
use crate::assets::texture_manager;
use wgpu::wgt::TextureViewDescriptor;
fn create_lut(resources: &Resources) {
    let mut texture_manager = resources.get_mut::<texture_manager::TextureManager>().unwrap();
    let texture =  {
        let device = resources.get::<wgpu::Device>().unwrap(); 
        let queue = resources.get::<wgpu::Queue>().unwrap();
    
        let lut = Arc::new(renderer::skybox_manager::BRDFLUTBuilder::build(&device, &queue));
        let view = Arc::new(lut.create_view(&TextureViewDescriptor::default()));
        let extent = wgpu::Extent3d {height: lut.width(), width: lut.width(), ..Default::default()};
        let format = lut.format();
        Arc::new(crate::assets::texture::Texture {inner: lut, view,  extent, _format: format })
    };

    texture_manager.textures.insert("lut".into(), texture);
} */
