use crate::model_reader;
use crate::prelude::*;
use crate::scene::Scene;

use legion::Resources;

pub struct App {
    pub(super) renderer: Option<Renderer>,
    pub(super) last_render_time: instant::Instant,
    pub(super) camera: Camera, // remove me
    pub(super) mouse_pressed: Option<winit::event::MouseButton>,
    pub current_scene: Scene,
    pub resources: Resources,
}

impl Default for App {
    fn default() -> Self {
        let camera = Camera::default(); // remove me
        let mut current_scene = Scene::default();
        let mut resources = Resources::default();

        // Add camera to Ecs
        crate::entities::camera::create(&mut current_scene.world, Camera::default());

        Self {
            renderer: None,
            last_render_time: instant::Instant::now(),
            camera, // remove me
            mouse_pressed: None,
            current_scene,
            resources,
        }
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
