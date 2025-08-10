use std::time::Instant;

use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use wgpu::{Device, Queue, TextureFormat};
use winit::window::Window;

pub struct ImguiState {
    pub context: imgui::Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub clear_color: wgpu::Color,
    pub demo_open: bool,
    pub last_frame: Instant,
    pub last_cursor: Option<MouseCursor>,
}

impl ImguiState {
    pub fn setup_imgui(
        device: &Device,
        queue: &Queue,
        window: &Window,
        format: TextureFormat,
    ) -> Self {
        let mut context = imgui::Context::create();
        let mut platform = WinitPlatform::new(&mut context);
        let hidpi_factor = window.scale_factor();

        platform.attach_window(
            context.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        context.set_ini_filename(None);

        let font_size = (13.0 * hidpi_factor) as f32;

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

        let renderer_config = RendererConfig {
            texture_format: format,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut context, device, queue, renderer_config);
        let last_frame = Instant::now();
        let last_cursor = None;
        let demo_open = true;

        Self {
            context,
            platform,
            renderer,
            clear_color,
            demo_open,
            last_frame,
            last_cursor,
        }
    }

    pub fn update_ui(&mut self, window: &Window, resources: &mut legion::Resources) {
        let delta_s = self.last_frame.elapsed();
        let now = Instant::now();
        self.context
            .io_mut()
            .update_delta_time(now - self.last_frame);
        self.last_frame = now;

        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("failed_to prepare frame");

        let ui = self.context.frame();
        {
            let window = ui.window("Hello world");
            window
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("Hello world!");
                    ui.text("This...is...imgui-rs on WGPU!");
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });
            let window = ui.window("Hello too");
            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([400.0, 200.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text(format!("Frametime: {delta_s:?}"));
                });
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
