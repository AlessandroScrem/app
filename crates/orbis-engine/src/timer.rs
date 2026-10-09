#![allow(dead_code)]
use std::time::{Duration, Instant};

pub struct Timer {
    clock: Instant,        // timer since application start
    delta_time: f32,       //time since last frame
    elapsed_time: f32,     //timer since last update
    frame_time: f32,       //time taken to render last frame
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
    #[allow(dead_code)]
    pub fn frametime(&mut self) -> f32 {
        let frametime = self.clock.elapsed().as_secs_f32() - self.elapsed_time;
        self.frame_time = frametime * 1000.0;
        frametime
    }

    /// Update the timer with the given frametime.
    /// Clamps the delta_time to FIXED_TIMESTEP to avoid large timesteps.
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

            // Esegui la callback
            callback();
        }
    }
}
