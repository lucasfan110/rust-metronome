use clap::Parser;
use crossterm::{
    ExecutableCommand,
    terminal::{Clear, ClearType},
};
use metronome::{
    data::{MetronomeData, TEMPO_RANGE, TempoType, TimeSignature, beat::MetronomeBeatTracker},
    sound::{MetronomeSound, MetronomeSoundType},
};
use std::{
    io,
    sync::{Arc, RwLock, mpsc},
    thread,
    time::Duration,
};
use tempo_measurer::TempoMeasurer;
use ui::Ui;
use user_input::UserInput;

mod metronome;
mod tempo_measurer;
mod ui;
mod user_input;

const TICK_LENGTH: Duration = Duration::from_micros(1);

/// Number of taps needed before tap mode changes the tempo
const TAPS_NEEDED: usize = 4;

/// A metronome written in Rust. Once entered, you can type in commands to change the
/// various settings within the metronome, such as the tempo, the time signature, the
/// subdivision, etc. Once entered the metronome, type in `help` and press enter to see
/// more detailed information about all the commands
#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// The tempo for the metronome, in beats per minute. Cannot be less than 10,
    /// or greater than 400
    #[arg(default_value_t = 60, value_parser = clap::value_parser!(u16).range(TEMPO_RANGE))]
    tempo: u16,

    /// The time signature for the metronome, in the format of a fraction. For example,
    /// `4/4` or `6/8`
    #[arg(default_value_t = TimeSignature::default())]
    time_signature: TimeSignature,

    /// The tempo type for the metronome. By default, it's quarter note equals,
    /// but for time signatures like `6/8`, it'll be dotted quarter equals, and
    /// for time signatures like `2/2`, it'll be half-note equals.
    #[arg(value_enum, short, long)]
    tempo_type: Option<TempoType>,

    /// The subdivision for the metronome, in terms of numbers. For example,
    /// `2` represents splitting a beat into 2
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..))]
    subdivision: u8,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let metronome_data = Arc::new(RwLock::new(MetronomeData::new(&cli)));

    let mut ui = Ui::new(Arc::clone(&metronome_data));
    let metronome_sound = MetronomeSound::new()?;
    let mut metronome_beat_tracker = MetronomeBeatTracker::new(Arc::clone(&metronome_data));
    // The sink variable returned from using the play audio function, saved
    // so that it continues playing because if it's dropped, then the audio stops
    let mut _sink;

    let (sender, receiver) = mpsc::channel::<UserInput>();

    // The thread for input
    let value = Arc::clone(&metronome_data);
    thread::spawn(move || -> anyhow::Result<()> {
        let metronome_data = value;
        let mut tempo_measurer = TempoMeasurer::new();

        loop {
            let mut input_str = String::new();

            io::stdin().read_line(&mut input_str)?;
            io::stdout().execute(Clear(ClearType::All))?;

            let input_str = input_str.trim();

            if metronome_data.read().unwrap().tap_mode {
                if matches!(input_str, "quit" | "q") {
                    metronome_data.write().unwrap().tap_mode = false;
                    tempo_measurer.clear();
                    sender.send(UserInput::Resume)?;
                    continue;
                }

                tempo_measurer.tap();

                if tempo_measurer.num_tapped() >= TAPS_NEEDED {
                    sender.send(UserInput::SetTempoDirect(tempo_measurer.calculate_tempo()))?;
                } else {
                    sender.send(UserInput::Clear)?;
                }

                continue;
            }

            let user_input = input_str.parse::<UserInput>();
            if let Ok(user_input) = user_input {
                sender.send(user_input)?;
            }
        }
    });

    io::stdout().execute(Clear(ClearType::All))?;

    loop {
        thread::sleep(TICK_LENGTH);

        let (time_signature_is_eighths, beat_info, should_play_subdivision_beat, is_paused) = {
            let m = metronome_data.read().unwrap();
        
            (
                m.time_signature_is_eighths(),
                m.beat_info,
                m.subdivision_setting.should_play_subdivision_beat(
                    m.beat_info,
                    m.time_signature_is_eighths(),
                    m.subdivision() > 1,
                ),
                m.is_paused,
            )
        };

        if !is_paused && metronome_beat_tracker.should_play_beat() {
            if should_play_subdivision_beat {
                _sink = metronome_sound.play(MetronomeSoundType::from_beat_info(
                    beat_info,
                    time_signature_is_eighths,
                ))?;
            }

            if metronome_beat_tracker.is_downbeat() {
                ui.render()?;
            }

            metronome_beat_tracker.move_to_next_subdivided_beat();
        }

        // If got a message from the input thread
        if let Ok(message) = receiver.try_recv() {
            metronome_data.write().unwrap().execute(&message);

            if let UserInput::Pause = message {
                // Subtract last beat time by a huge amount, so when the user resumes,
                // there is no awkward wait, especially for slower tempo
                metronome_beat_tracker.offset_beat_timestamp();
            }

            ui.render()?;
        }
    }
}
