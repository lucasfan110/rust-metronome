use crate::timer::render_tracker::TimerRenderTracker;
use clap::Parser;
use crossterm::{
    ExecutableCommand,
    terminal::{Clear, ClearType},
};
use input_thread::start_input_thread;
use metronome::{
    data::{
        MetronomeData, SUBDIVISION_RANGE, TEMPO_RANGE, TempoType, TimeSignature,
        beat::{MetronomeBeatTracker, accent::get_metronome_beat_accent},
    },
    sound::play_metronome_sound,
};
use std::{
    io,
    sync::{Arc, RwLock, mpsc},
    thread,
    time::Duration,
};
use timer::play_timer_alarm;
use ui::Ui;
use user_input::UserInput;

mod input_thread;
mod metronome;
mod tempo_measurer;
mod timer;
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
    #[arg(default_value_t = 60, value_parser = clap::value_parser!(i32).range(TEMPO_RANGE))]
    tempo: i32,

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
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(i32).range(SUBDIVISION_RANGE))]
    subdivision: i32,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let metronome_data = Arc::new(RwLock::new(MetronomeData::new(&cli)));

    let mut ui = Ui::new(Arc::clone(&metronome_data));
    let mut metronome_beat_tracker = MetronomeBeatTracker::new(Arc::clone(&metronome_data));
    let mut timer_render_tracker = TimerRenderTracker::new(Arc::clone(&metronome_data));

    let mut _metronome_sound_data;
    let mut _timer_alarm_sound_data: Option<(rodio::OutputStream, rodio::Sink)> = None;

    let (sender, receiver) = mpsc::channel::<UserInput>();

    // The thread for input
    start_input_thread(Arc::clone(&metronome_data), sender);

    io::stdout().execute(Clear(ClearType::All))?;

    loop {
        thread::sleep(TICK_LENGTH);

        if !metronome_data.read().unwrap().is_paused && metronome_beat_tracker.should_play_beat() {
            metronome_beat_tracker.move_to_next_subdivided_beat();

            let d = metronome_data.read().unwrap();
            let should_play_subdivision_beat = d.subdivision_setting.should_play_subdivision_beat(
                d.beat_info,
                d.time_signature_is_eighths(),
                d.subdivision() > 1,
            );

            if should_play_subdivision_beat {
                _metronome_sound_data =
                    play_metronome_sound(get_metronome_beat_accent(d.beat_accents(), d.beat_info))?;
            }

            drop(d);

            if metronome_beat_tracker.is_downbeat() {
                ui.render()?;
            }
        }

        if timer_render_tracker.should_render_timer() {
            ui.render()?;
            timer_render_tracker.just_rendered();

            let d = metronome_data.read().unwrap();
            let timer = d.timer.as_ref().unwrap();

            if timer.time_remaining().is_zero() {
                _timer_alarm_sound_data = Some(play_timer_alarm()?);
            }
        }

        // If got a message from the input thread
        if let Ok(message) = receiver.try_recv() {
            metronome_data.write().unwrap().execute(&message);

            match message {
                UserInput::Pause => metronome_beat_tracker.offset_beat_timestamp(),
                UserInput::StopTimer => _timer_alarm_sound_data = None,
                _ => {}
            }

            ui.render()?;
        }
    }
}
