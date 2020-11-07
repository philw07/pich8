use std::time::Instant;

pub struct FpsCounter {
    start: Instant,
    frames: u32,
    fps: f64,
    previous_fps: f64,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            frames: 0,
            fps: 0.0,
            previous_fps: 0.0,
        }
    }

    pub fn tick(&mut self) -> f64 {
        self.frames += 1;

        // Update fps value every half second
        if self.start.elapsed().as_secs() >= 1 {
            let new_fps = (self.frames as f64 / self.start.elapsed().as_nanos() as f64) * 1_000_000_000.0;
            self.previous_fps = self.fps;
            self.fps = if self.previous_fps > 0.0 { 0.33 * self.previous_fps + 0.33 * self.fps + 0.34 * new_fps } else { new_fps };

            self.start = Instant::now();
            self.frames = 0;
        }

        self.fps
    }
}