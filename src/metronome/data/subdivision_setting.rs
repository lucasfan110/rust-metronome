use anyhow::anyhow;
use std::{fmt, str::FromStr};

use super::beat::BeatInfo;

const PLAY_SUBDIVISION_CHAR: char = 'x';
const SILENCE_SUBDIVISION_CHAR: char = '-';

#[derive(Debug, Clone, Default)]
pub struct SubdivisionSetting {
    pub play_beat: Vec<bool>,
}

impl FromStr for SubdivisionSetting {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut play_beat: Vec<bool> = Vec::new();

        for c in s.chars() {
            if !matches!(c, PLAY_SUBDIVISION_CHAR | SILENCE_SUBDIVISION_CHAR) {
                return Err(anyhow!(
                    "Subdivision setting can only contain \"{}\" or \"{}\"!",
                    PLAY_SUBDIVISION_CHAR,
                    SILENCE_SUBDIVISION_CHAR
                ));
            }

            play_beat.push(c == PLAY_SUBDIVISION_CHAR);
        }

        Ok(Self { play_beat })
    }
}

impl fmt::Display for SubdivisionSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for p in self.play_beat.iter().copied() {
            write!(
                f,
                "{}",
                if p {
                    PLAY_SUBDIVISION_CHAR
                } else {
                    SILENCE_SUBDIVISION_CHAR
                }
            )?;
        }

        Ok(())
    }
}

impl SubdivisionSetting {
    pub fn should_play_subdivision_beat(
        &self,
        beat_info: BeatInfo,
        time_signature_is_eighths: bool,
        has_subdivision: bool,
    ) -> bool {
        if beat_info == (0, 0) {
            return true;
        }
        let play_beat = if time_signature_is_eighths && !has_subdivision {
            self.play_beat.get(beat_info.current_beat as usize % 3)
        } else {
            self.play_beat.get(beat_info.subdivided_beat as usize)
        };

        play_beat.copied().unwrap_or(true)
    }
}
