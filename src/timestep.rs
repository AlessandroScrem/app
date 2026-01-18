use std::time::{Duration, Instant};
#[derive(Clone)]
pub struct Timestep {
    time: Duration,     // durata frame corrente (smoothed)
    last: Instant,      // istante ultimo frame
    fps: f32,           // fps istantaneo (calcolato ogni secondo)
    count: f32,         // conteggio frame nel secondo corrente
    timer: f32,         // timer accumulato per aggiornare fps
    avg_time: Duration, // media mobile del timestep
    avg_fps: f32,       // FPS medio coerente con avg_time
    samples: u32,       // numero campioni accumulati
    started: bool,      // indica se è partito il conteggio effettivo
    warmup: f32,        // accumula tempo prima dell'avvio
}

impl Timestep {
    pub fn new() -> Self {
        Self {
            time: Duration::from_secs_f32(0.0),
            last: Instant::now(),
            fps: 0.0,
            count: 0.0,
            timer: 0.0,
            avg_time: Duration::from_secs_f32(0.0),
            avg_fps: 0.0,
            samples: 0,
            started: false,
            warmup: 0.0,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now - self.last;
        self.last = now;

        // smoothing
        const MEDIAN: f32 = 0.9;
        const VARIANT: f32 = 0.1;
        let blended = self.time.as_secs_f32() * MEDIAN + dt.as_secs_f32() * VARIANT;
        self.time = Duration::from_secs_f32(blended);

        // warmup
        if !self.started {
            self.warmup += dt.as_secs_f32();
            if self.warmup >= 1.0 {
                self.started = true;
                self.last = Instant::now();
                self.time = Duration::from_secs_f32(0.0);
                self.timer = 0.0;
                self.count = 0.0;
                self.samples = 0;
                self.avg_time = Duration::from_secs_f32(0.0);
            }
            return;
        }

        // aggiorna media mobile del timestep
        self.samples += 1;
        let avg_secs = (self.avg_time.as_secs_f32() * ((self.samples - 1) as f32)
            + dt.as_secs_f32())
            / (self.samples as f32);
        self.avg_time = Duration::from_secs_f32(avg_secs);

        // calcola FPS coerente con il frame time medio
        self.avg_fps = 1.0 / self.avg_time.as_secs_f32().max(1e-6);

        // FPS "istantaneo" aggiornato ogni secondo (solo per info)
        self.count += 1.0;
        self.timer += dt.as_secs_f32();
        if self.timer >= 1.0 {
            self.fps = self.count;
            self.count = 0.0;
            self.timer = 0.0;
        }
    }

    pub fn delta(&self) -> Duration {
        self.time
    }

    pub fn average(&self) -> Duration {
        self.avg_time
    }

    pub fn average_fps(&self) -> u32 {
        self.avg_fps as u32
    }

    #[allow(dead_code)]
    fn fps(&self) -> u32 {
        self.fps as u32
    }

    #[allow(dead_code)]
    fn started(&self) -> bool {
        self.started
    }
}
