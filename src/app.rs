use crate::model_reader;
use crate::prelude::*;

pub struct App {
    pub(super) renderer: Option<Renderer>,
    pub(super) last_render_time: instant::Instant,
    pub(super) camera: Camera,
    pub(super) mouse_pressed: Option<winit::event::MouseButton>,
    picked_file: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        const FOV: cgmath::Deg<f32> = cgmath::Deg::<f32>(45.0);
        let camera = Camera::new(FOV, 1.0, 0.1, 100.0);

        Self {
            renderer: None,
            last_render_time: instant::Instant::now(),
            camera,
            mouse_pressed: None,
            picked_file: None,
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
    pub fn update_gui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if ui.button("Open file…").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.picked_file = Some(path.display().to_string());
            }
        }

        if self.picked_file.is_some() {
            egui::Window::new("File selezionato")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if let Some(path) = &self.picked_file {
                        ui.label(format!("Hai selezionato:\n{}", path));
                    } else {
                        ui.label("Nessun file selezionato.");
                    }

                    if ui.button("Chiudi").clicked() {
                        let meshes =
                            model_reader::load_gltf(self.picked_file.as_ref().unwrap()).unwrap();
                        println!("Loaded {} meshes", meshes.len());

                        self.picked_file = None;
                    }
                });
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
