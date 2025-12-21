use crate::timer::Timer;
use crate::{Cli, metronome::help_menu::print_help, user_input::UserInput};
use TempoType::*;
use anyhow::anyhow;
use beat::BeatInfo;
use clap::ValueEnum;
use std::{
    fmt::{self, Display, Formatter},
    ops::RangeInclusive,
    process,
    str::FromStr,
    time::Duration,
};
use subdivision_setting::SubdivisionSetting;

pub mod beat;
pub mod subdivision_setting;

pub const TEMPO_MIN: u16 = 10;
pub const TEMPO_MAX: u16 = 400;
pub const TEMPO_RANGE: RangeInclusive<i64> = (TEMPO_MIN as i64)..=(TEMPO_MAX as i64);

pub const fn is_tempo_valid(tempo: u16) -> bool {
    TEMPO_MIN <= tempo && tempo <= TEMPO_MAX
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
            QuarterNote => 1.0,
            EighthNote => 0.5,
            SixteenthNote => 0.25,
            HalfNote => 2.0,
            WholeNote => 4.0,
            DottedQuarter => 1.5,
            DottedHalf => 3.0,
            DottedWhole => 6.0,
        }
    }

    /// A function that gets the default tempo type based on the time signature.
    /// If a tempo type ends in `8`, like `3/8` or `6/8`, then the tempo type by
    /// default is dotted quarter note equals. And if it ends in a `2`, like cut
    /// time `2/2`, then tempo is half-note equals by default. Otherwise, it defaults
    /// to quarter note equals
    fn get_default_based(time_signature: TimeSignature) -> Self {
        match time_signature.1 {
            2 => HalfNote,
            8 => DottedQuarter,
            _ => QuarterNote,
        }
    }
}

impl Display for TempoType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = match self {
            QuarterNote => "Quarter Note",
            EighthNote => "Eighth Note",
            SixteenthNote => "Sixteenth Note",
            HalfNote => "Half Note",
            WholeNote => "Whole Note",
            DottedQuarter => "Dotted Quarter Note",
            DottedHalf => "Dotted Half Note",
            DottedWhole => "Dotted Whole Note",
        };

        write!(f, "{}", string)
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
            _ => Err(anyhow!("Invalid tempo type {}!", s)),
        }
    }
}

/// The time signature, with the first u8 representing the number of beats in a
/// measure, and the second u8 representing what each beat is equivalent to
#[derive(Debug, Clone, Copy)]
pub struct TimeSignature(pub u8, pub u8);

impl Default for TimeSignature {
    fn default() -> Self {
        Self(4, 4)
    }
}

impl FromStr for TimeSignature {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numbers: Vec<&str> = s.split('/').collect();

        if numbers.len() != 2 {
            return Err(anyhow!(
                "Invalid time signature! Example time signature: 4/4 or 6/8"
            ));
        }

        let numerator: u8 = numbers[0].parse()?;
        let denominator: u8 = numbers[1].parse()?;

        if numerator == 0 {
            return Err(anyhow!("Numerator on the time signature cannot be 0!"));
        }

        if denominator == 0 || !denominator.is_power_of_two() {
            return Err(anyhow!(
                "Denominator on the time signature must be a power of 2 and cannot be 0!"
            ));
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
    pub beat_info: BeatInfo,
    tempo_type: TempoType,
    time_signature: TimeSignature,
    subdivision: u8,
    pub subdivision_setting: SubdivisionSetting,
    duration_per_subdivided_beat: Duration,
    pub is_paused: bool,
    pub tap_mode: bool,
    pub timer: Option<Timer>,
}

// Getters and setters
impl MetronomeData {
    fn recalculate_duration_per_subdivided_beat(&mut self) {
        // Convert the tempo type to the note length, with quarter note being length
        // of 1
        let note_length = self.tempo_type.to_note_length();
        // A bit of magic, but basically, it's the tempo multiplied by the amplifier
        // depending on what note value a tempo is equal to and the current time signature
        let new_tempo = self.tempo as f64 * self.time_signature.1 as f64 / (4.0 / note_length);

        self.duration_per_subdivided_beat =
            Duration::from_secs_f64(60.0 / new_tempo / self.subdivision as f64);
    }

    pub fn change_tempo(&mut self, tempo: u16) {
        self.tempo = tempo.clamp(TEMPO_MIN, TEMPO_MAX);
        self.beat_info.reset();
        self.recalculate_duration_per_subdivided_beat();
    }

    pub fn change_tempo_type(&mut self, tempo_type: TempoType) {
        self.tempo_type = tempo_type;
        self.change_tempo(self.tempo);
    }

    pub fn change_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
        self.change_tempo_type(TempoType::get_default_based(time_signature));
    }

    pub fn change_subdivision(&mut self, subdivision: u8) {
        self.subdivision = subdivision;
        self.recalculate_duration_per_subdivided_beat();
        self.beat_info.reset();
    }

    pub fn tempo(&self) -> u16 {
        self.tempo
    }

    pub fn tempo_type(&self) -> TempoType {
        self.tempo_type
    }

    pub fn subdivision(&self) -> u8 {
        self.subdivision
    }

    pub fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }
}

impl MetronomeData {
    pub fn new(cli: &Cli) -> Self {
        let tempo_type = cli
            .tempo_type
            .unwrap_or(TempoType::get_default_based(cli.time_signature));

        let mut new_value = Self {
            tempo: cli.tempo,
            time_signature: cli.time_signature,
            tempo_type,
            subdivision: cli.subdivision,
            subdivision_setting: SubdivisionSetting::default(),
            beat_info: (cli.time_signature.0 - 1, cli.subdivision - 1).into(),
            duration_per_subdivided_beat: Duration::ZERO,
            is_paused: false,
            tap_mode: false,
            timer: None,
        };

        new_value.recalculate_duration_per_subdivided_beat();
        new_value
    }

    pub fn time_signature_is_eighths(&self) -> bool {
        self.time_signature.1 == 8
    }

    pub fn execute(&mut self, user_input: &UserInput) {
        use UserInput::*;

        match user_input {
            Pause => self.is_paused = true,
            Resume => {
                self.is_paused = false;
                self.beat_info.reset();
            }
            Help => print_help(),
            Clear => {}
            SetTempo(tempo_str) => match tempo_str.parse::<u16>() {
                Ok(tempo) if is_tempo_valid(tempo) => {
                    self.change_tempo(tempo);
                }
                _ => println!(
                    "Invalid tempo `{}`! Must be a valid whole number between {}-{}!",
                    tempo_str, TEMPO_MIN, TEMPO_MAX
                ),
            },
            SetTempoDirect(tempo) => self.change_tempo(*tempo),
            SetTimeSignature(time_signature_str) => {
                match time_signature_str.parse::<TimeSignature>() {
                    Ok(time_signature) => self.change_time_signature(time_signature),
                    Err(_) => println!("Invalid time signature `{}`!", time_signature_str),
                }
            }
            SetTempoType(tempo_type_str) => match tempo_type_str.parse::<TempoType>() {
                Ok(tempo_type) => self.change_tempo_type(tempo_type),
                Err(_) => println!("Invalid tempo type {}!", tempo_type_str),
            },
            SetSubdivision(subdivision_str) => {
                self.change_subdivision(subdivision_str.parse::<u8>().unwrap_or(1).max(1))
            }
            SetSubdivisionSetting(subdivision_setting_str) => {
                match subdivision_setting_str.parse::<SubdivisionSetting>() {
                    Ok(subdivision_setting) => self.subdivision_setting = subdivision_setting,
                    Err(err) => println!(
                        "Invalid subdivision setting \"{}\"! (Error: {})",
                        subdivision_setting_str, err
                    ),
                }
            }
            Tap => {
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
            SetTimer(duration) => match Timer::from_str(duration) {
                Ok(t) => self.timer = Some(t),
                Err(_) => {
                    println!("Invalid timer string! Format: HH:MM:SS, hour and minutes optional")
                }
            },
            StopTimer => self.timer = None,
        };

        if self.is_paused {
            println!("PAUSED!");
        }
        if self.tap_mode {
            println!("TAP MODE. Press enter for each beat. Enter `q` to exit.");
        }
    }
}
