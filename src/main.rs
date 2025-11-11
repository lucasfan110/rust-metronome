use clap::{CommandFactory, Parser};
use crossterm::ExecutableCommand;
use crossterm::terminal::{Clear, ClearType};
use metronome_data::tempo_measurer::TempoMeasurer;
use metronome_data::{MetronomeData, TEMPO_RANGE, TempoType, TimeSignature};
use metronome_sound::{MetronomeSound, MetronomeSoundType};
use std::io;
use std::sync::{Arc, RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use ui::Ui;
use user_input::UserInput;

mod metronome_data;
mod metronome_sound;
mod ui;
mod user_input;

const TICK_LENGTH: Duration = Duration::from_micros(1);

#[derive(Parser)]
#[command(version, about, long_about)]
struct Cli {
    /// The tempo for the metronome, in beats per minute. Cannot be less than 20,
    /// or greater than 400
    #[arg(value_parser = clap::value_parser!(u16).range(TEMPO_RANGE))]
    tempo: u16,

    /// The time signature for the metronome, in the format of a fraction. For example,
    /// `4/4` or `6/8`
    #[arg(default_value_t = String::from("4/4"))]
    time_signature: String,

    /// The tempo type for the metronome. By default it's quarter note equals,
    /// but for time signatures like `6/8`, it'll be dotted quarter equals, and
    /// for time signatures like `2/2`, it'll be half note equals.
    #[arg(value_enum, short, long)]
    tempo_type: Option<TempoType>,

    /// The subdivision for the metronome, in terms of numbers. For example,
    /// `2` represents splitting a beat into 2
    #[arg(short, long, default_value_t = 1)]
    subdivision: u8,

    /// The subdivision setting. It gives you an ability to customize what beats
    /// are played and what beats are silent for the subdivision. Use `0` for silent
    /// beat, `1` for a beat that plays, and `-` for a beat to slur over.
    ///
    /// For example, if the subdivision is `4`, then `--subdivision-setting 01-1` will mean silent
    /// the first beat, the second and third beat is slurred together, and the
    /// fourth beat is played, by itself.
    #[arg(short = 'u', long)]
    subdivision_setting: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Check if time signature is valid
    if let Err(err) = cli.time_signature.parse::<TimeSignature>() {
        Cli::command()
            .error(clap::error::ErrorKind::InvalidValue, err)
            .exit();
    }

    let metronome_data = Arc::new(RwLock::new(MetronomeData::new(&cli)));

    // Track the time stamp when the last metronome beat was played. Did this whole
    // minus thing so that there is no awkward wait for the first metronome beat when
    // the program first starts.
    let mut last_beat_time = Instant::now() - Duration::from_secs(1_000);

    let ui = Ui::new(Arc::clone(&metronome_data));
    let metronome_sound = MetronomeSound::new()?;
    // The sink variable that is returned from using the play audio function, saved
    // so that it continues playing because if it's dropped then the audio stops
    let mut _sink;

    let mut tempo_measurer = TempoMeasurer::new();

    let (sender, receiver) = mpsc::channel::<UserInput>();

    // The thread for input
    let value = Arc::clone(&metronome_data);
    thread::spawn(move || -> anyhow::Result<()> {
        let metronome_data = value;

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

                if tempo_measurer.num_tapped() >= 4 {
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

        let should_play_beat = {
            let metronome_data = metronome_data.read().unwrap();
            !metronome_data.is_paused
                && Instant::now() - last_beat_time >= metronome_data.duration_per_beat()
        };

        if should_play_beat {
            last_beat_time = Instant::now();

            let metronome_sound_type = {
                let metronome_data = metronome_data.read().unwrap();

                // Depending on which beat the metronome is on, it plays a different
                // metronome sound
                MetronomeSoundType::from_beat(
                    metronome_data.beat,
                    metronome_data.time_signature_is_eighths(),
                )
            };

            _sink = metronome_sound.play(metronome_sound_type)?;
            ui.render()?;

            metronome_data.write().unwrap().next_beat()
        }

        // If got a message from the input thread
        if let Ok(message) = receiver.try_recv() {
            message.execute(Arc::clone(&metronome_data));
            ui.render()?;
        }
    }
}
