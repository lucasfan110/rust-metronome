use crate::{Cli, metronome_sound::MetronomeSoundType, user_input::UserInput};
use clap::ValueEnum;
use std::{
    fmt::{self, Display, Formatter},
    ops::RangeInclusive,
    process,
    str::FromStr,
    time::Duration,
};

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

fn print_help() {
    println!("Commands: ");
    println!("pause, p: Pause the metronome");
    println!("resume, r: Resume the metronome");
    println!("quit, q: Exit the metronome");
    println!("help, h: Print help");
    println!("clear, c: Clear the screen");
    println!("tempo, t <TEMPO>: Set the tempo of the metronome");
    println!("\tExample: t 60");
    println!("time <TIME_SIGNATURE>: Set the time signature of the metronome");
    println!("tempo-type, tt <TEMPO_TYPE>: Set the tempo type of the metronome");
    println!(
        "\tExample: `tt dotted-quarter` changes the current tempo type \
        from whatever to dotted quarter note equals"
    );
    println!("subdivision, s <SUBDIVISION>: Set the subdivision of the metronome");
    println!(
        "tap: Enters tap mode. Press return for each beat, and after 4 taps, the \
        tempo of the metronome will automatically change to the tapped tempo. Press \
        `q` to stop"
    )
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
    duration_per_subdivided_beat: Duration,
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

        let mut new_value = Self {
            tempo: cli.tempo,
            time_signature,
            tempo_type,
            subdivision: cli.subdivision,
            beat: 0,
            duration_per_beat: Duration::ZERO,
            duration_per_subdivided_beat: Duration::ZERO,
            is_paused: false,
            tap_mode: false,
        };

        new_value.recalculate_duration_per_beat();
        new_value
    }

    pub fn duration_per_beat(&self) -> Duration {
        self.duration_per_beat
    }

    fn recalculate_duration_per_subdivided_beat(&mut self) {
        self.duration_per_subdivided_beat = self.duration_per_beat.div_f64(self.subdivision as f64);
    }

    fn recalculate_duration_per_beat(&mut self) {
        self.duration_per_beat =
            get_duration_per_beat(self.tempo, self.tempo_type, self.time_signature);
        self.recalculate_duration_per_subdivided_beat();
    }

    fn reset_beat(&mut self) {
        self.beat = 0;
    }

    pub fn set_tempo(&mut self, tempo: u16) {
        self.tempo = tempo.clamp(TEMPO_MIN, TEMPO_MAX);
        self.reset_beat();
        self.recalculate_duration_per_beat();
    }

    pub fn tempo(&self) -> u16 {
        self.tempo
    }

    pub fn set_tempo_type(&mut self, tempo_type: TempoType) {
        self.tempo_type = tempo_type;
        // Make sure the tempo stays the "same", with the same amount of beat per
        // second relative to what note value a tempo equals to
        let new_tempo = (self.tempo as f64
            * (self.tempo_type.to_note_length() / tempo_type.to_note_length()))
        .round() as u16;
        // Still set the tempo because rounding error may cause slight change in
        // the duration per beat
        self.set_tempo(new_tempo);
    }

    pub fn tempo_type(&self) -> TempoType {
        self.tempo_type
    }

    pub fn set_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
        self.set_tempo_type(TempoType::get_default_based(time_signature));
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

    pub fn set_subdivision(&mut self, subdivision: u8) {
        self.subdivision = subdivision;
        self.recalculate_duration_per_subdivided_beat();
    }

    pub fn next_beat(&mut self) {
        self.beat = (self.beat + 1) % self.time_signature().0;
    }

    pub fn get_current_sound_type(&self) -> MetronomeSoundType {
        // Depending on which beat the metronome is on, it plays a different
        // metronome sound
        MetronomeSoundType::from_beat(self.beat, self.time_signature_is_eighths())
    }

    pub fn execute(&mut self, user_input: &UserInput) {
        use UserInput::*;

        match user_input {
            Pause => {
                println!("PAUSED!");
                self.is_paused = true
            }
            Resume => {
                self.is_paused = false;
                self.beat = 0;
            }
            Help => print_help(),
            Clear => {}
            SetTempo(tempo_str) => match tempo_str.parse::<u16>() {
                Ok(tempo) if is_tempo_valid(tempo) => {
                    self.set_tempo(tempo);
                }
                _ => println!(
                    "Invalid tempo `{}`! Must be a valid whole number between {}-{}!",
                    tempo_str, TEMPO_MIN, TEMPO_MAX
                ),
            },
            SetTempoDirect(tempo) => self.set_tempo(*tempo),
            SetTimeSignature(time_signature_str) => {
                match time_signature_str.parse::<TimeSignature>() {
                    Ok(time_signature) => self.set_time_signature(time_signature),
                    Err(_) => println!("Invalid time signature `{}`!", time_signature_str),
                }
            }
            SetTempoType(tempo_type_str) => match tempo_type_str.parse::<TempoType>() {
                Ok(tempo_type) => self.set_tempo_type(tempo_type),
                Err(_) => println!("Invalid tempo type {}!", tempo_type_str),
            },
            SetSubdivision(subdivision_str) => match subdivision_str.parse::<u8>() {
                Ok(subdivision) if subdivision >= 1 => self.set_subdivision(subdivision),
                _ => println!(
                    "Invalid subdivision {}! Must be a whole number that is at least 1!",
                    subdivision_str
                ),
            },
            Tap => {
                println!("TAP MODE. Press enter for each beat. Enter `q` to exit.");

                self.tap_mode = true;
                self.is_paused = true;
            }
            Quit => {
                println!("Goodbye!");
                process::exit(0);
            }
            Unknown(command) => {
                println!("Unknown command `{}`!", command);
            }
        }
    }

    pub fn duration_per_subdivided_beat(&self) -> Duration {
        self.duration_per_subdivided_beat
    }
}
