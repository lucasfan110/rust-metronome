use crate::metronome::data::TimeSignature;

use super::BeatInfo;

#[derive(Debug, Clone, Copy)]
pub enum MetronomeBeatAccent {
    Accented = 0,
    Beat,
    Subdivision,
}

pub fn get_metronome_beat_accent(
    metronome_beat_accents: &[MetronomeBeatAccent],
    beat_info: BeatInfo,
) -> MetronomeBeatAccent {
    if beat_info.subdivided_beat != 0 {
        MetronomeBeatAccent::Subdivision
    } else {
        metronome_beat_accents[beat_info.current_beat as usize]
    }
}

fn eighths_beat_accents(time_signature: TimeSignature) -> Vec<MetronomeBeatAccent> {
    let mut beat_accents = vec![MetronomeBeatAccent::Subdivision; time_signature.0 as usize];
    beat_accents[0] = MetronomeBeatAccent::Accented;

    match time_signature.0 {
        7 => {
            beat_accents[3] = MetronomeBeatAccent::Beat;
            beat_accents[5] = MetronomeBeatAccent::Beat;
        }
        _ => {
            for i in (0..time_signature.0).step_by(3).skip(1) {
                beat_accents[i as usize] = MetronomeBeatAccent::Beat;
            }
        }
    }
    beat_accents
}

pub fn get_beat_accents_from_time_signature(
    time_signature: TimeSignature,
) -> Vec<MetronomeBeatAccent> {
    match time_signature.1 {
        8 => eighths_beat_accents(time_signature),
        _ => {
            let mut beat_accents = vec![MetronomeBeatAccent::Beat; time_signature.0 as usize];
            beat_accents[0] = MetronomeBeatAccent::Accented;
            beat_accents
        }
    }
}
