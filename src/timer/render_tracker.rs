use crate::metronome::data::MetronomeData;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub struct TimerRenderTracker {
    metronome_data: Arc<RwLock<MetronomeData>>,
    last_rerender_timestamp: Instant,
}

impl TimerRenderTracker {
    pub fn new(metronome_data: Arc<RwLock<MetronomeData>>) -> Self {
        Self {
            metronome_data,
            last_rerender_timestamp: Instant::now(),
        }
    }

    pub fn should_render_timer(&self) -> bool {
        if self.metronome_data.read().unwrap().timer.is_none() {
            return false;
        }

        self.last_rerender_timestamp.elapsed() >= Duration::from_secs(1)
    }

    pub fn just_rendered(&mut self) {
        self.last_rerender_timestamp = Instant::now();
    }
}
