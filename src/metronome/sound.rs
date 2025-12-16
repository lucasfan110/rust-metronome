use std::io::Cursor;

use super::data::beat::BeatInfo;

static METRONOME_SOUNDS: &[&[u8]] = &[
    include_bytes!("../audio/beat1.mp3"),
    include_bytes!("../audio/beat2.mp3"),
    include_bytes!("../audio/beat3.mp3"),
];

#[derive(Debug, Clone, Copy)]
pub enum MetronomeSoundType {
    Accented = 0,
    Beat,
    Subdivision,
}

impl MetronomeSoundType {
    fn get_audio_bytes(self) -> &'static [u8] {
        METRONOME_SOUNDS[self as usize]
    }

    /// Get the appropriate metronome sound to play based on the current beat of
    /// the metronome. If the time signature ends in 8, then change it up so that
    /// it gives like a subdivided feel
    pub fn from_beat_info(beat_info: BeatInfo, is_eighths_time_signature: bool) -> Self {
        match is_eighths_time_signature {
            true => {
                if beat_info == (0, 0) {
                    Self::Accented
                } else if beat_info.current_beat.is_multiple_of(3) && beat_info.subdivided_beat == 0
                {
                    Self::Beat
                } else {
                    Self::Subdivision
                }
            }
            false => {
                if beat_info == (0, 0) {
                    Self::Accented
                } else if beat_info.subdivided_beat == 0 {
                    Self::Beat
                } else {
                    Self::Subdivision
                }
            }
        }
    }
}

pub struct MetronomeSound {
    stream_handle: rodio::OutputStream,
}

impl MetronomeSound {
    pub fn new() -> Result<Self, rodio::StreamError> {
        Ok(Self {
            stream_handle: rodio::OutputStreamBuilder::open_default_stream()?,
        })
    }

    pub fn play(&self, metronome_sound_type: MetronomeSoundType) -> anyhow::Result<rodio::Sink> {
        Ok(rodio::play(
            self.stream_handle.mixer(),
            Cursor::new(metronome_sound_type.get_audio_bytes()),
        )?)
    }
}
