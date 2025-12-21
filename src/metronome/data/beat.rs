use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use super::MetronomeData;

fn modulo_increment(num: &mut u8, modulo_by: u8) {
    *num += 1;
    *num %= modulo_by;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BeatInfo {
    pub current_beat: u8,
    pub subdivided_beat: u8,
}

impl From<(u8, u8)> for BeatInfo {
    fn from(value: (u8, u8)) -> Self {
        Self {
            current_beat: value.0,
            subdivided_beat: value.1,
        }
    }
}

impl PartialEq<(u8, u8)> for BeatInfo {
    fn eq(&self, other: &(u8, u8)) -> bool {
        self.current_beat == other.0 && self.subdivided_beat == other.1
    }
}

impl BeatInfo {
    pub fn next_subdivided_beat(&mut self, num_beats: u8, num_subdivided_beats: u8) {
        modulo_increment(&mut self.subdivided_beat, num_subdivided_beats);

        if self.subdivided_beat == 0 {
            modulo_increment(&mut self.current_beat, num_beats);
        }
    }

    pub fn reset(&mut self) {
        *self = Self::from((0, 0));
    }
}

pub struct MetronomeBeatTracker {
    metronome_data: Arc<RwLock<MetronomeData>>,
    last_beat_timestamp: Instant,
}

fn instant_now_offset() -> Instant {
    const DURATION_OFFSET: Duration = Duration::from_secs(1_000);
    Instant::now() - DURATION_OFFSET
}

impl MetronomeBeatTracker {
    pub fn offset_beat_timestamp(&mut self) {
        self.last_beat_timestamp = instant_now_offset();
    }

    pub fn new(metronome_data: Arc<RwLock<MetronomeData>>) -> Self {
        Self {
            metronome_data,
            last_beat_timestamp: instant_now_offset(),
        }
    }

    pub fn move_to_next_subdivided_beat(&mut self) {
        let MetronomeData {
            time_signature,
            subdivision,
            ..
        } = *self.metronome_data.read().unwrap();

        self.metronome_data
            .write()
            .unwrap()
            .beat_info
            .next_subdivided_beat(time_signature.0, subdivision);
        self.last_beat_timestamp = Instant::now();
    }

    pub fn should_play_beat(&self) -> bool {
        let duration_per_subdivided_beat = self
            .metronome_data
            .read()
            .unwrap()
            .duration_per_subdivided_beat;

        self.last_beat_timestamp.elapsed() >= duration_per_subdivided_beat
    }

    pub fn is_downbeat(&self) -> bool {
        self.metronome_data
            .read()
            .unwrap()
            .beat_info
            .subdivided_beat
            == 0
    }
}
