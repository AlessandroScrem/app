use std::time::Instant;

use super::DeltaTime;
use crate::input::Input;
use crate::input::MouseButton;
use crate::model_reader;
use crate::prelude::*;
use crate::scene::Scene;

use legion::Resources;
use legion::Schedule;

pub fn camera_orbit_system() -> impl legion::systems::Runnable {
    use legion::IntoQuery;
    use legion::SystemBuilder;
    use legion::Write;

    SystemBuilder::new("Camera Orbit")
        .read_resource::<DeltaTime>()
        .read_resource::<crate::input::Input>()
        .with_query(<Write<Camera>>::query())
        .build(|_cmd, world, (_, input), camera_query | {
            for camera in camera_query.iter_mut(world) {
                if input.is_mouse_button_down(MouseButton::Left) {
                    let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
                    camera.orbit(delta);
                }

                if input.is_mouse_button_down(MouseButton::Middle) {
                    let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
                    camera.pan(delta);
                }

                if let Some(delta) = input.mouse_wheel_movement {
                    camera.zoom(delta.y);
                }
            }
        })
}

pub struct App {
    pub(super) window: Option<std::sync::Arc<winit::window::Window>>,
    pub(super) renderer: Option<Renderer>,
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
            renderer: None,
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
            .add_system(camera_orbit_system())
            .build();

        let _delta = self.resources.get_mut::<DeltaTime>().unwrap();
        
        self.render_schedule = crate::systems::create_render_schedule_builder();
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

    /*
    pub fn update_gui(&self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        struct DisplayPoint3(pub cgmath::Point3<f32>);
        impl std::fmt::Display for DisplayPoint3 {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "({:.2}, {:.2}, {:.2})", self.0.x, self.0.y, self.0.z)
            }
        }
         if ui.button("Open file…").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                // self.load_gltf(path.as_path());
            }
        }

        ui.label(format!(
            "Camera position: {}",
            DisplayPoint3(self.camera.get_position())
        ));
        let mut fov: f32 = self.camera.get_fov().0;
        if ui
            .add(egui::Slider::new(&mut fov, 0.1..=179.0).text("Fov"))
            .changed()
        {
            // self.camera.set_fov(cgmath::Deg(fov));
        };
    }
    */
}
