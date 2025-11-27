use crossterm::{
    QueueableCommand, cursor,
    style::{Print, PrintStyledContent, Stylize},
};
use std::{
    fmt::{self, Write as FmtWrite},
    io::{self, Write},
    mem,
    sync::{Arc, RwLock},
};

use crate::metronome::data::MetronomeData;

const SCREEN_TEXT_CAPACITY: usize = 256;

fn get_beat_to_print(beat_index: u8, is_eighths_time_signature: bool) -> String {
    let beat = match is_eighths_time_signature {
        true => {
            if beat_index == 0 {
                "X"
            } else if beat_index.is_multiple_of(3) {
                "x"
            } else {
                "."
            }
        }
        false => {
            if beat_index == 0 {
                "X"
            } else {
                "x"
            }
        }
    };

    String::from(beat)
}

#[derive(Debug, Clone)]
pub struct Ui {
    screen_text: String,
    pub metronome_data: Arc<RwLock<MetronomeData>>,
}

impl Ui {
    pub fn new(metronome_data: Arc<RwLock<MetronomeData>>) -> Self {
        Self {
            metronome_data,
            screen_text: String::with_capacity(SCREEN_TEXT_CAPACITY),
        }
    }

    pub fn render(&mut self) -> io::Result<()> {
        io::stdout()
            .queue(cursor::SavePosition)?
            .queue(cursor::MoveTo(0, 0))?;

        self.write_info_text().unwrap();
        self.write_metronome_beat_text().unwrap();

        io::stdout()
            .queue(Print(mem::replace(
                &mut self.screen_text,
                String::with_capacity(SCREEN_TEXT_CAPACITY),
            )))?
            .queue(cursor::RestorePosition)?
            .flush()?;

        Ok(())
    }

    fn write_info_text(&mut self) -> fmt::Result {
        let metronome_data = self.metronome_data.read().unwrap();
        let subdivision = if metronome_data.subdivision() == 1 {
            String::from("None")
        } else {
            metronome_data.subdivision().to_string()
        };

        writeln!(
            self.screen_text,
            "Tempo: {} = {}\t\tTime Signature = {}\t\tSubdivision = {}",
            metronome_data.tempo_type(),
            metronome_data.tempo(),
            metronome_data.time_signature(),
            subdivision,
        )
    }

    fn write_metronome_beat_text(&mut self) -> fmt::Result {
        write!(self.screen_text, "[    ")?;

        let (beats_per_measure, beat, is_eighths_time_signature) = {
            let metronome_data = self.metronome_data.read().unwrap();
            (
                metronome_data.time_signature().0,
                metronome_data.beat,
                metronome_data.time_signature_is_eighths(),
            )
        };

        for i in 0..beats_per_measure {
            let beat_to_print = get_beat_to_print(i, is_eighths_time_signature);

            if i == beat {
                write!(
                    self.screen_text,
                    "{}",
                    PrintStyledContent(beat_to_print.italic().blue())
                )?;
            } else {
                write!(self.screen_text, "{}", beat_to_print)?;
            }

            write!(self.screen_text, " ")?;
        }

        write!(self.screen_text, "   ]")?;

        Ok(())
    }
}
