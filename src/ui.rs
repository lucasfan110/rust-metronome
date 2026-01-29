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

use crate::metronome::data::{
    MetronomeData,
    beat::{BeatInfo, accent::get_metronome_beat_accent},
};

const SCREEN_TEXT_CAPACITY: usize = 256;

const BEAT_CHAR: [char; 3] = ['X', 'x', '.'];

// fn get_beat_to_print(beat_index: i32, is_eighths_time_signature: bool) -> char {
//     match is_eighths_time_signature {
//         true => {
//             if beat_index == 0 {
//                 'X'
//             } else if beat_index % 3 == 0 {
//                 'x'
//             } else {
//                 '.'
//             }
//         }
//         false => {
//             if beat_index == 0 {
//                 'X'
//             } else {
//                 'x'
//             }
//         }
//     }
// }

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
        self.write_timer_text().unwrap();

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

        let data = self.metronome_data.read().unwrap();
        let beats_per_measure = data.time_signature().0;

        for i in 0..beats_per_measure {
            // let beat_to_print = get_beat_to_print(i, data.time_signature_is_eighths());
            let current_beat_accent =
                get_metronome_beat_accent(data.beat_accents(), BeatInfo::from((i, 0)));
            let beat_to_print = BEAT_CHAR[current_beat_accent as usize];

            if i == data.beat_info.current_beat {
                write!(self.screen_text, "{}", beat_to_print.italic().blue())?;
            } else {
                write!(self.screen_text, "{}", beat_to_print)?;
            }

            write!(self.screen_text, " ")?;
        }

        writeln!(self.screen_text, "   ]")?;

        Ok(())
    }

    fn write_timer_text(&mut self) -> fmt::Result {
        if let Some(ref timer) = self.metronome_data.read().unwrap().timer {
            writeln!(self.screen_text, "TIMER: {}", timer.time_remaining_str())?;
        }

        Ok(())
    }
}
