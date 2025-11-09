use std::env;
use std::fs::File;

#[derive(Clone, Copy)]
pub enum MetronomeSoundType {
    Accented = 1,
    Beat,
    Subdivision,
}

impl MetronomeSoundType {
    fn get_file_path(self) -> String {
        // By default the metronome sound files should be located in
        // %HOME_DIR%/rust-metronome/audio
        let home_dir = env::home_dir().expect("Failed to find home_dir");
        let home_dir = home_dir
            .to_str()
            .expect("Failed to convert home_dir to string");

        format!(
            "{}\\rust-metronome\\audio\\beat{}.mp3",
            home_dir, self as i32
        )
    }

    /// Get the appropriate metronome sound to play based on the current beat of
    /// the metronome. If the time signature ends in 8, then change it up so that
    /// it gives like a subdivided feel
    pub fn from_beat(beat: u8, is_eighths_time_signature: bool) -> Self {
        match is_eighths_time_signature {
            true => {
                if beat == 0 {
                    Self::Accented
                } else if beat.is_multiple_of(3) {
                    Self::Beat
                } else {
                    Self::Subdivision
                }
            }
            false => {
                if beat == 0 {
                    Self::Accented
                } else {
                    Self::Beat
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
        let file_name = metronome_sound_type.get_file_path();
        let beat_audio = File::open(file_name)?;

        Ok(rodio::play(self.stream_handle.mixer(), beat_audio)?)
    }
}
