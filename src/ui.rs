use crate::metronome_data::MetronomeData;
use crossterm::QueueableCommand;
use crossterm::cursor;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

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

pub struct Ui {
    pub metronome_data: Arc<Mutex<MetronomeData>>,
}

impl Ui {
    pub fn new(metronome_data: Arc<Mutex<MetronomeData>>) -> Self {
        Self { metronome_data }
    }

    pub fn render(&self) -> io::Result<()> {
        io::stdout()
            .queue(cursor::SavePosition)?
            .queue(cursor::MoveTo(0, 0))?;

        self.print_info()?;
        self.print_metronome_beat()?;

        io::stdout().queue(cursor::RestorePosition)?.flush()?;

        Ok(())
    }

    fn print_info(&self) -> io::Result<()> {
        let metronome_data = self.metronome_data.lock().unwrap();
        let subdivision = if metronome_data.subdivision() == 1 {
            String::from("None")
        } else {
            metronome_data.subdivision().to_string()
        };

        io::stdout().queue(Print(format!(
            "Tempo: {} = {}\t\tTime Signature = {}\t\tSubdivision = {}\n",
            metronome_data.tempo_type(),
            metronome_data.tempo(),
            metronome_data.time_signature(),
            subdivision,
        )))?;

        Ok(())
    }

    fn print_metronome_beat(&self) -> io::Result<()> {
        io::stdout().queue(Print("[    "))?;

        let (beats_per_measure, beat, is_eighths_time_signature) = {
            let metronome_data = self.metronome_data.lock().unwrap();
            (
                metronome_data.time_signature().0,
                metronome_data.beat,
                metronome_data.time_signature_is_eighths(),
            )
        };

        for i in 0..beats_per_measure {
            let beat_to_print = get_beat_to_print(i, is_eighths_time_signature);

            if i == beat {
                io::stdout().queue(Print(PrintStyledContent(beat_to_print.italic().blue())))?;
            } else {
                io::stdout().queue(Print(beat_to_print))?;
            }

            io::stdout().queue(Print(" "))?;
        }

        io::stdout().queue(Print("   ]\n"))?;

        Ok(())
    }
}
