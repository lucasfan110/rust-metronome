use crate::Cli;
use clap::ValueEnum;
use std::fmt::{self, Display, Formatter};
use std::ops::RangeInclusive;
use std::str::FromStr;
use std::time::Duration;

pub mod tempo_measurer;

pub const TEMPO_MIN: u16 = 20;
pub const TEMPO_MAX: u16 = 400;
pub const TEMPO_RANGE: RangeInclusive<i64> = (TEMPO_MIN as i64)..=(TEMPO_MAX as i64);

pub fn is_tempo_valid(tempo: u16) -> bool {
    (TEMPO_MIN..=TEMPO_MAX).contains(&tempo)
}

/// Get the duration per beat based on the tempo, the tempo type, and the time signature.
/// Returns the `Duration` struct which indicates how long per beat
fn get_duration_per_beat(
    tempo: u16,
    tempo_type: TempoType,
    time_signature: TimeSignature,
) -> Duration {
    // Convert the tempo type to the note length, with quarter note being length
    // of 1
    let note_length = tempo_type.to_note_length();
    // A bit of magic, but basically, it's the tempo multiplied by the amplifier
    // depending on what note value a tempo is equal to and the current time signature
    let new_tempo = tempo as f64 * time_signature.1 as f64 / (4.0 / note_length);

    Duration::from_secs_f64(60.0 / new_tempo)
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
pub enum TempoType {
    QuarterNote,
    EighthNote,
    SixteenthNote,
    HalfNote,
    WholeNote,
    DottedQuarter,
    DottedHalf,
    DottedWhole,
}

impl TempoType {
    fn to_note_length(self) -> f64 {
        match self {
            TempoType::QuarterNote => 1.0,
            TempoType::EighthNote => 0.5,
            TempoType::SixteenthNote => 0.25,
            TempoType::HalfNote => 2.0,
            TempoType::WholeNote => 4.0,
            TempoType::DottedQuarter => 1.5,
            TempoType::DottedHalf => 3.0,
            TempoType::DottedWhole => 6.0,
        }
    }

    /// A function that gets the default tempo type based on the time signature.
    /// If a tempo type ends in `8`, like `3/8` or `6/8`, then the tempo type by
    /// default is dotted quarter note equals. And if it ends in a `2`, like cut
    /// time `2/2`, then tempo is half note equals by default. Otherwise it defaults
    /// to quarter note equals
    fn get_default_based(time_signature: TimeSignature) -> Self {
        match time_signature.1 {
            2 => TempoType::HalfNote,
            8 => TempoType::DottedQuarter,
            _ => TempoType::QuarterNote,
        }
    }
}

impl Display for TempoType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            TempoType::QuarterNote => "Quarter Note",
            TempoType::EighthNote => "Eighth Note",
            TempoType::SixteenthNote => "Sixteenth Note",
            TempoType::HalfNote => "Half Note",
            TempoType::WholeNote => "Whole Note",
            TempoType::DottedQuarter => "Dotted Quarter Note",
            TempoType::DottedHalf => "Dotted Half Note",
            TempoType::DottedWhole => "Dotted Whole Note",
        };

        write!(f, "{}", str)
    }
}

impl FromStr for TempoType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use TempoType::*;

        match s {
            "quarter-note" => Ok(QuarterNote),
            "eighth-note" => Ok(EighthNote),
            "sixteenth-note" => Ok(SixteenthNote),
            "half-note" => Ok(HalfNote),
            "whole-note" => Ok(WholeNote),
            "dotted-quarter" => Ok(DottedQuarter),
            "dotted-half" => Ok(DottedHalf),
            "dotted-whole" => Ok(DottedWhole),
            _ => Err(anyhow::anyhow!("Invalid tempo type {}!", s)),
        }
    }
}

/// The time signature, with the first u8 representing the amount of beats in a
/// measure, and the second u8 representing what each beat is equivalent to
#[derive(Clone, Copy)]
pub struct TimeSignature(pub u8, pub u8);

impl FromStr for TimeSignature {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numbers: Vec<&str> = s.split('/').collect();

        if numbers.len() != 2 {
            anyhow::bail!("Invalid time signature!");
        }

        let numerator: u8 = numbers[0].parse()?;
        let denominator: u8 = numbers[1].parse()?;

        if numerator == 0 {
            anyhow::bail!("Numerator on the time signature cannot be 0!");
        }

        if denominator == 0 || !denominator.is_power_of_two() {
            anyhow::bail!(
                "Denominator on the time signature must be a power of 2 and cannot be 0!"
            );
        }

        Ok(Self(numerator, denominator))
    }
}

impl Display for TimeSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

pub struct MetronomeData {
    tempo: u16,
    duration_per_beat: Duration,
    pub beat: u8,
    tempo_type: TempoType,
    time_signature: TimeSignature,
    subdivision: u8,
    pub is_paused: bool,
    pub tap_mode: bool,
}

impl MetronomeData {
    pub fn new(cli: &Cli) -> Self {
        let time_signature: TimeSignature = cli.time_signature.parse().unwrap();

        let tempo_type = cli
            .tempo_type
            .unwrap_or(TempoType::get_default_based(time_signature));

        Self {
            tempo: cli.tempo,
            time_signature,
            tempo_type,
            subdivision: cli.subdivision,
            beat: time_signature.0 - 1,
            duration_per_beat: get_duration_per_beat(cli.tempo, tempo_type, time_signature),
            is_paused: false,
            tap_mode: false,
        }
    }

    pub fn duration_per_beat(&self) -> Duration {
        self.duration_per_beat
    }

    pub fn set_tempo(&mut self, tempo: u16) {
        self.tempo = tempo.clamp(TEMPO_MIN, TEMPO_MAX);
        self.beat = 0;
        self.duration_per_beat =
            get_duration_per_beat(self.tempo, self.tempo_type, self.time_signature);
    }

    pub fn tempo(&self) -> u16 {
        self.tempo
    }

    pub fn set_tempo_type(&mut self, tempo_type: TempoType) {
        self.tempo = (self.tempo as f64
            * (self.tempo_type.to_note_length() / tempo_type.to_note_length()))
            as u16;
        self.tempo_type = tempo_type;
        self.duration_per_beat =
            get_duration_per_beat(self.tempo, self.tempo_type, self.time_signature);
    }

    pub fn tempo_type(&self) -> TempoType {
        self.tempo_type
    }

    pub fn set_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
        self.tempo_type = TempoType::get_default_based(time_signature);
        self.duration_per_beat =
            get_duration_per_beat(self.tempo, self.tempo_type, self.time_signature);
        self.beat = 0;
    }

    pub fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    pub fn time_signature_is_eighths(&self) -> bool {
        self.time_signature.1 == 8
    }

    pub fn subdivision(&self) -> u8 {
        self.subdivision
    }

    pub fn next_beat(&mut self) {
        self.beat = (self.beat + 1) % self.time_signature().0;
    }
}
