use std::time::{Duration, Instant};

pub struct TempoMeasurer {
    timestamps: Vec<Instant>,
}

impl TempoMeasurer {
    pub fn new() -> Self {
        Self {
            timestamps: Vec::new(),
        }
    }

    pub fn tap(&mut self) {
        self.timestamps.push(Instant::now());
    }

    pub fn calculate_tempo(&self) -> u16 {
        let total_duration = self
            .timestamps
            .windows(2)
            .fold(Duration::ZERO, |acc, timestamps| {
                acc + (timestamps[1] - timestamps[0])
            });

        let secs_per_beat = total_duration.as_secs_f64() / (self.timestamps.len() - 1) as f64;
        (1.0 / secs_per_beat * 60.0) as u16
    }

    pub fn num_tapped(&self) -> usize {
        self.timestamps.len()
    }

    pub fn clear(&mut self) {
        self.timestamps.clear();
    }
}
