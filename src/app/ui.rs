use crate::{UiSnapshot, engine::RunningApp};

use super::app::App;

impl App {
    pub fn update_uilayer(&mut self, runtime: &mut RunningApp) {
        let uilayer = &mut runtime.uilayer;
        let window = &runtime.window;
        let counter_trait = &runtime.gpu_cache;

        let snapshot = UiSnapshot::from_world(
            &self.current_scene.world,
            self.selected,
            &self.asset_mgr,
            &self.camera,
            &self.globals,
            &runtime.imgui_render, //resolver trait
            counter_trait,         // internalcounter trait
            None,
        );

        // Main operation: update_ui
        let mut events = uilayer.build(window, snapshot);
        self.domain_events.queue.append(&mut events);
    }
}
