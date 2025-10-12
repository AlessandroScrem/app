use crate::input::Input;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use crate::Globals;
use crate::prelude::imgui_tools::ImguiState;
use crate::prelude::*;
use crate::scene::Scene;

use legion::Resources;
use legion::Schedule;
use legion::systems::Builder;

pub struct Timer {
    pub clock: Instant,    // timer since application start
    pub delta_time: f32,   //time since last frame
    pub elapsed_time: f32, //timer since last update
    pub frame_time: f32,   //time taken to render last frame
    last_trigger: Instant, // last time the every() callback was triggered
}

impl Timer {
    pub const FIXED_TIMESTEP: f32 = 1.0 / 60.0; //minimum timestep (to avoid leg)
    pub fn new() -> Self {
        Self {
            clock: Instant::now(),
            delta_time: 0.0,
            elapsed_time: 0.0,
            frame_time: 0.0,
            last_trigger: Instant::now(),
        }
    }

    /// Returns the time in seconds since the last call to frametime() and updates the internal timer.
    pub fn frametime(&mut self) -> f32 {
        let frametime = self.clock.elapsed().as_secs_f32() - self.elapsed_time;
        self.frame_time = frametime * 1000.0;
        frametime
    }

    /// Update the timer with the given frametime.
    /// Clamps the delta_time to FIXED_TIMESTEP to avoid large timesteps.
    pub fn tick(&mut self, frametime: f32) -> f32 {
        self.delta_time = f32::min(frametime, Self::FIXED_TIMESTEP);
        self.elapsed_time += self.delta_time;
        self.delta_time
    }

    /// Iterator that yields fixed timestep steps until the current frametime is covered.
    /// This can be used to run fixed timestep updates in a variable timestep environment.
    ///
    /// # Example
    /// ```ignore
    /// let mut timer = super::Timer::new();
    /// let frametime = timer.frametime();
    /// for dt in timer.tick_step_iter() {
    ///     // Run fixed timestep update with dt
    /// }
    /// ```
    pub fn tick_step_iter(&mut self) -> impl Iterator<Item = f32> + '_ {
        let mut remaining = self.frametime();
        std::iter::from_fn(move || {
            if remaining > 0.0 {
                let dt = remaining.min(Self::FIXED_TIMESTEP);
                remaining -= dt;
                Some(self.tick(dt))
            } else {
                None
            }
        })
    }

    /// Trigger a callback every `interval` duration.
    /// The callback is called once for each interval that has passed since the last trigger.
    /// This function should be called every frame, typically in the main loop.
    /// # Example
    /// ```ignore
    /// use std::time::Duration;
    /// let mut timer = Timer::new();
    /// loop {
    ///   timer.trigger_every(Duration::from_secs(1), || {
    ///     println!("This prints every second");
    /// });
    /// }
    /// ```
    ///  
    pub fn trigger_every<F>(&mut self, interval: Duration, mut callback: F)
    where
        F: FnMut(),
    {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_trigger);

        if elapsed >= interval {
            // Calcola quanti intervalli completi sono passati
            let steps = elapsed.as_nanos() / interval.as_nanos();
            self.last_trigger += interval * steps as u32;

            // Esegui la callback una volta per ogni intervallo passato
            for step in 0..steps {
                trace!("triggerd: step: {} at: {:?}", step, self.clock.elapsed());
                callback();
            }
        }
    }
}

pub struct App {
    pub(super) window: Option<Arc<winit::window::Window>>,
    pub current_scene: Scene,
    pub resources: Resources,
    pub timer: Timer,
    pub size: winit::dpi::PhysicalSize<u32>,
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
            timer: Timer::new(),
            size: winit::dpi::PhysicalSize::new(1280, 1024),
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
