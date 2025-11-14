use clap::{CommandFactory, Parser};
use crossterm::{
    ExecutableCommand,
    terminal::{Clear, ClearType},
};
use metronome_data::{
    MetronomeData, TEMPO_RANGE, TempoType, TimeSignature, tempo_measurer::TempoMeasurer,
};
use metronome_sound::{MetronomeSound, MetronomeSoundType};
use std::{
    io,
    sync::{Arc, RwLock, mpsc},
    thread,
    time::{Duration, Instant},
};
use ui::Ui;
use user_input::UserInput;

mod metronome_data;
mod metronome_sound;
mod ui;
mod user_input;

const TICK_LENGTH: Duration = Duration::from_micros(1);

fn should_play_beat(metronome_data: &MetronomeData, last_beat_time: Instant) -> bool {
    !metronome_data.is_paused && last_beat_time.elapsed() >= metronome_data.duration_per_beat()
}

fn should_play_subdivided_beat(
    metronome_data: &MetronomeData,
    last_subdivided_beat_time: Instant,
) -> bool {
    !metronome_data.is_paused
        && last_subdivided_beat_time.elapsed() >= metronome_data.duration_per_subdivided_beat()
}

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
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..))]
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
    let mut last_subdivided_beat_time = Instant::now();

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

        if should_play_beat(&metronome_data.read().unwrap(), last_beat_time) {
            last_beat_time = Instant::now();
            last_subdivided_beat_time = Instant::now();

            let metronome_sound_type = metronome_data.read().unwrap().get_current_sound_type();
            _sink = metronome_sound.play(metronome_sound_type)?;
            ui.render()?;

            metronome_data.write().unwrap().next_beat()
        } else if should_play_subdivided_beat(
            &metronome_data.read().unwrap(),
            last_subdivided_beat_time,
        ) {
            last_subdivided_beat_time = Instant::now();
            _sink = metronome_sound.play(MetronomeSoundType::Subdivision)?;
        }

        // If got a message from the input thread
        if let Ok(message) = receiver.try_recv() {
            metronome_data.write().unwrap().execute(&message);

            if let UserInput::Pause = message {
                // Subtract last beat time by a huge amount so when the user resumes
                // there is no awkward wait, especially for slower tempo
                last_beat_time -= Duration::from_secs(1_000);
            }

            ui.render()?;
        }
    }
}
