use std::time::Instant;

pub struct DurationWatch {
    prev: Instant,
    now: Instant,
    reset: bool,
}

impl DurationWatch {
    pub fn measure_as_sec(&mut self) -> f32 {
        if self.reset {
            self.now = Instant::now();
        }
        self.reset = false;
        (self.now - self.prev).as_secs_f32()
    }

    pub fn reset(&mut self) {
        self.reset = true;
        self.prev = self.now;
    }
}

impl Default for DurationWatch {
    fn default() -> Self {
        DurationWatch {
            prev: Instant::now(),
            now: Instant::now(),
            reset: true,
        }
    }
}
