use crate::metronome_data::{
    MetronomeData, TEMPO_MAX, TEMPO_MIN, TempoType, TimeSignature, is_tempo_valid,
};
use UserInput::*;
use std::{
    process,
    str::FromStr,
    sync::{Arc, Mutex},
};

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
    println!(
        "tap: Enters tap mode. Press return for each beat, and after 4 taps, the \
        tempo of the metronome will automatically change to the tapped tempo. Press \
        `q` to stop"
    )
}

pub enum UserInput {
    Pause,
    Resume,
    Quit,
    Help,
    Clear,
    Tap,
    SetTempo(String),
    SetTimeSignature(String),
    SetTempoType(String),
    Unknown(String),
}

impl FromStr for UserInput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowercase = s.to_lowercase();
        let args: Vec<&str> = lowercase.split(" ").collect();

        if args.is_empty() {
            return Ok(Self::Unknown(String::new()));
        }

        let get_nth_arg =
            |index: usize| -> String { args.get(index).copied().unwrap_or_default().to_string() };

        Ok(match args[0] {
            "pause" | "p" => Pause,
            "resume" | "r" => Resume,
            "quit" | "q" => Quit,
            "help" | "h" => Help,
            "clear" | "c" => Clear,
            "tempo" | "t" => SetTempo(get_nth_arg(1)),
            "time" => SetTimeSignature(get_nth_arg(1)),
            "tempo-type" | "tt" => SetTempoType(get_nth_arg(1)),
            "tap" => Tap,
            command => Unknown(command.to_string()),
        })
    }
}

impl UserInput {
    pub fn execute(&self, metronome_data: Arc<Mutex<MetronomeData>>) {
        let mut metronome_data = metronome_data.lock().unwrap();

        match self {
            Pause => {
                println!("PAUSED!");
                metronome_data.is_paused = true
            }
            Resume => {
                metronome_data.is_paused = false;
                metronome_data.beat = metronome_data.time_signature().0 - 1;
            }
            Help => print_help(),
            Clear => {}
            SetTempo(tempo_str) => match tempo_str.parse::<u16>() {
                Ok(tempo) if is_tempo_valid(tempo) => {
                    metronome_data.set_tempo(tempo);
                }
                _ => println!(
                    "Invalid tempo `{}`! Must be a valid whole number between {}-{}!",
                    tempo_str, TEMPO_MIN, TEMPO_MAX
                ),
            },
            SetTimeSignature(time_signature_str) => {
                match time_signature_str.parse::<TimeSignature>() {
                    Ok(time_signature) => metronome_data.set_time_signature(time_signature),
                    Err(_) => println!("Invalid time signature `{}`!", time_signature_str),
                }
            }
            SetTempoType(tempo_type_str) => match tempo_type_str.parse::<TempoType>() {
                Ok(tempo_type) => metronome_data.set_tempo_type(tempo_type),
                Err(_) => println!("Invalid tempo type {}!", tempo_type_str),
            },
            Tap => {
                println!("TAP MODE. Press enter for each beat. Press `q` to exit.");

                metronome_data.tap_mode = true;
                metronome_data.is_paused = true;
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
}
