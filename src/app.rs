use std::time::Instant;

use crate::input::Input;
use crate::input::MouseButton;
use crate::model_reader;
use crate::prelude::*;
use crate::scene::Scene;
use super::DeltaTime;

use cgmath::Vector3;
use cgmath::Zero;
use legion::Resources;
use legion::Schedule;



#[derive(Debug)]
pub struct CameraData {
    pub yaw: f32,
    pub pitch: f32,
    pub position: Vector3<f32>,
}

impl Default for CameraData {
    fn default() -> Self {
        Self { yaw: Default::default(), pitch: Default::default(), position: Vector3::zero() }
    }
}

impl CameraData {
    pub fn update_view(&mut self, eye: Vec3, center: Vec3, up: Vec3) {
        // Stub function for updating view matrix
    }
}


// Use your own Vec3 (from glam, nalgebra, etc.)
pub type Vec3 = Vector3<f32>;

pub fn camera_orbit_system() -> impl legion::systems::Runnable {
    use legion::IntoQuery;
    use legion::SystemBuilder;
    use legion::Write;

    SystemBuilder::new("Camera Orbit")
        .read_resource::<DeltaTime>()
        .read_resource::<crate::input::Input>()
        .with_query(<Write<CameraData>>::query())
        .build(
            |_cmd, world, (delta_time, input), camera_query| {
                for camera in camera_query.iter_mut(world) {
                    if !input.is_mouse_button_down(MouseButton::Left) {
                        continue;
                    }
                                    
                    println!("mouse button down");
                    camera.yaw += input.mouse_delta.x * 0.5 * delta_time.0;
                    camera.pitch += input.mouse_delta.y * 0.5 * delta_time.0;

                    camera.pitch = camera
                        .pitch
                        .max(-std::f32::consts::FRAC_PI_2 + 0.0001)
                        .min(std::f32::consts::FRAC_PI_2 - 0.0001);

                    let eye = Vec3::new(0.0, 0.0, 0.0)
                        + (5.0
                            * Vec3::new(
                                camera.yaw.sin() * camera.pitch.cos(),
                                camera.pitch.sin(),
                                camera.yaw.cos() * camera.pitch.cos(),
                            ));

                    camera.position = eye;
                    camera.update_view(eye, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
                }
            },
        )
}

pub struct App {
    pub(super) window: Option<std::sync::Arc<winit::window::Window>>,
    pub(super) renderer: Option<Renderer>,
    pub(super) camera: Camera, // remove me
    pub(super) mouse_pressed: Option<winit::event::MouseButton>,
    pub current_scene: Scene,
    pub resources: Resources,

    pub clock: Instant,
    pub fixed_timestep: f32,
    pub elapsed_time: f32,
    /// Time last frame took.
    pub frame_time: f32,
    /// Current delta time.
    pub delta_time: f32,
    pub last_frame: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            renderer: None,
            camera: Camera::default(), // remove me
            mouse_pressed: None,
            current_scene: Scene::default(),
            resources: Resources::default(),

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
        // crate::entities::camera::create(&mut self.current_scene.world, Camera::default());
        self.current_scene.world.push((CameraData::default(), ));

        self.resources.insert(Input::new());
        self.resources.insert(DeltaTime(10.0));
        self.current_scene.schedule = Schedule::builder()
            .add_system(camera_orbit_system())
            .build();

        let delta = self.resources.get_mut::<DeltaTime>().unwrap();
        println!("delta time is {:?}", delta);

    }
}

struct DisplayPoint3(pub cgmath::Point3<f32>);

impl std::fmt::Display for DisplayPoint3 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2})", self.0.x, self.0.y, self.0.z)
    }
}

impl App {
    fn load_gltf(&mut self, path: &std::path::Path) {
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

    pub fn update_gui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        if ui.button("Open file…").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.load_gltf(path.as_path());
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
            self.camera.set_fov(cgmath::Deg(fov));
        };
    }
}
