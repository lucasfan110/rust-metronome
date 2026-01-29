use std::{
    io,
    sync::{Arc, RwLock, mpsc},
    thread,
};

use crossterm::{
    ExecutableCommand,
    terminal::{Clear, ClearType},
};

use crate::{
    TAPS_NEEDED, metronome::data::MetronomeData, tempo_measurer::TempoMeasurer,
    user_input::UserInput,
};

pub fn start_input_thread(
    metronome_data: Arc<RwLock<MetronomeData>>,
    sender: mpsc::Sender<UserInput>,
) {
    thread::spawn(move || -> anyhow::Result<()> {
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
}
