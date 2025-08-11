use std::time::Instant;

use cgmath::{Deg, Rad};
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use winit::window::Window;

pub struct ImguiState {
    pub context: imgui::Context,
    pub platform: WinitPlatform,
    pub clear_color: wgpu::Color,
    pub demo_open: bool,
    pub last_frame: Instant,
    pub last_cursor: Option<MouseCursor>,
}

impl ImguiState {
    pub fn create_imgui(window: &Window, resources: &mut legion::Resources) -> Self {
        let mut context = imgui::Context::create();
        let mut platform = WinitPlatform::new(&mut context);
        let hidpi_factor = window.scale_factor();

        platform.attach_window(
            context.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        context.set_ini_filename(None);

        let font_size = (9.0 * hidpi_factor) as f32;

        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let last_frame = Instant::now();
        let last_cursor = None;
        let demo_open = true;

        let renderer = {
            let device = resources.get::<wgpu::Device>().unwrap();
            let queue = resources.get::<wgpu::Queue>().unwrap();
            let format = resources
                .get::<wgpu::SurfaceConfiguration>()
                .unwrap()
                .format;
            let renderer_config = RendererConfig {
                texture_format: format,
                ..Default::default()
            };
            Renderer::new(&mut context, &device, &queue, renderer_config)
        };

        resources.insert(renderer);

        Self {
            context,
            platform,
            clear_color,
            demo_open,
            last_frame,
            last_cursor,
        }
    }

    pub fn update_ui(
        &mut self,
        window: &Window,
        world: &mut legion::World,
        resources: &mut legion::Resources,
    ) {
        let delta_s = self.last_frame.elapsed();
        let now = Instant::now();

        self.context
            .io_mut()
            .update_delta_time(now - self.last_frame);
        self.last_frame = now;

        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("failed_to prepare frame");

        use legion::query::IntoQuery;
        let mut camera_query = <&mut crate::camera::Camera>::query();
        let camera = camera_query.iter_mut(world).next();
        
        let ui = self.context.frame();
        {
            let window = ui.window("General info");
            window
                .size([300.0, 100.0], Condition::FirstUseEver)
                .position([0.0, 0.0], Condition::FirstUseEver)
                .build(|| {
                    ui.separator();
                    ui.text(format!("Frametime: {delta_s:?}"));
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });

            ui.camera_window(camera);
            
        }

        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(ui, window);
        }

        let draw_data: &DrawData = self.context.render();
        let owned = OwnedDrawData::from(draw_data);
        resources.insert(owned);
    }
}

use crate::camera::Camera;
pub trait UiCameraExt {
    fn camera_window(&self, camera: Option<&mut Camera>);
}

impl UiCameraExt for imgui::Ui {
    fn camera_window(&self, camera: Option<&mut Camera>) {
        let camera = match camera {
            Some(camera) => camera,
            None => return,
        };
        let window = self.window("Camera");
        window
            .size([300.0, 300.0], Condition::FirstUseEver)
            .position([0.0, 100.0], Condition::FirstUseEver)
            .build(|| {
                self.text(format!("Position: {:?}", camera.get_position()));
                self.text(format!("FocalPoint: {:?}", camera.get_focal_point()));
                self.text(format!(
                    "Yaw/Pitch: {:.1} {:.1}",
                    camera.get_yaw_pitch().0,
                    camera.get_yaw_pitch().1
                ));
                self.separator();

                let mut fov = Deg::from(camera.fov).0;
                if Drag::new("Fov")
                    .range(1.0f32, 179.0f32)
                    .speed(1.0)
                    .build(self, &mut fov)
                {
                    camera.fov = Rad(fov.to_radians());
                }

                let mut distance = camera.get_distance();
                if Drag::new("Distance")
                    .range(0f32, 10f32)
                    .speed(0.01)
                    .build(self, &mut distance)
                {
                    camera.set_distance(distance);
                }

                let mut near = camera.near;
                let mut far = camera.far;
                if DragRange::new("Near/Far")
                    .range(0.1, 100.0)
                    .speed(0.01)
                    .build(self, &mut near, &mut far)
                {
                    let near = near.max(0.1);
                    let far = far.max(near + 0.1);
                    camera.near = near;
                    camera.far = far;
                }
            });
    }
}
