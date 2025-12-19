use crossterm::{
    QueueableCommand, cursor,
    style::{Print, Stylize},
};
use std::{
    fmt::{self, Write as FmtWrite},
    io::{self, Write},
    mem,
    sync::{Arc, RwLock},
};

use crate::metronome::data::MetronomeData;

const SCREEN_TEXT_CAPACITY: usize = 256;

fn get_beat_to_print(beat_index: u8, is_eighths_time_signature: bool) -> char {
    match is_eighths_time_signature {
        true => {
            if beat_index == 0 {
                'X'
            } else if beat_index.is_multiple_of(3) {
                'x'
            } else {
                '.'
            }
        }
        false => {
            if beat_index == 0 {
                'X'
            } else {
                'x'
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ui {
    screen_text: String,
    metronome_data: Arc<RwLock<MetronomeData>>,
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

        let screen_text = mem::replace(
            &mut self.screen_text,
            String::with_capacity(SCREEN_TEXT_CAPACITY),
        );

        io::stdout()
            .queue(Print(screen_text))?
            .queue(cursor::RestorePosition)?
            .flush()?;

        Ok(())
    }

    fn write_info_text(&mut self) -> fmt::Result {
        let metronome_data = self.metronome_data.read().unwrap();

        write!(
            self.screen_text,
            "Tempo: {} = {}\t\tTime Signature = {}\t\tSubdivision = ",
            metronome_data.tempo_type(),
            metronome_data.tempo(),
            metronome_data.time_signature(),
        )?;

        if metronome_data.subdivision() <= 1 {
            write!(self.screen_text, "None")?;
        } else {
            write!(self.screen_text, "{}", metronome_data.subdivision())?;
        };

        if !metronome_data.subdivision_setting.play_beat.is_empty() {
            write!(
                self.screen_text,
                " ({})",
                metronome_data.subdivision_setting
            )?;
        }

        writeln!(self.screen_text)?;

        Ok(())
    }

    fn write_metronome_beat_text(&mut self) -> fmt::Result {
        write!(self.screen_text, "[    ")?;

        let (beats_per_measure, beat_info, is_eighths_time_signature) = {
            let metronome_data = self.metronome_data.read().unwrap();
            (
                metronome_data.time_signature().0,
                metronome_data.beat_info,
                metronome_data.time_signature_is_eighths(),
            )
        };

        for i in 0..beats_per_measure {
            let beat_to_print = get_beat_to_print(i, is_eighths_time_signature);

            if i == beat_info.current_beat {
                write!(self.screen_text, "{}", beat_to_print.italic().blue())?;
            } else {
                write!(self.screen_text, "{}", beat_to_print)?;
            }

            write!(self.screen_text, " ")?;
        }

        write!(self.screen_text, "   ]")?;

        Ok(())
    }
}
