use anyhow::anyhow;
use rodio::{Decoder, Source};
use std::fmt::Write;
use std::io::Cursor;
use std::str::FromStr;
use std::time::{Duration, Instant};

static ALARM_AUDIO_DATA: &[u8] = include_bytes!("./audio/timer-alarm.mp3");

pub mod render_tracker;

fn create_infinite_playback(audio_data: &'static [u8]) -> anyhow::Result<impl Source> {
    let source = Decoder::new(Cursor::new(audio_data))?;
    let source_looped = source.repeat_infinite();

    Ok(source_looped)
}

pub fn play_timer_alarm() -> anyhow::Result<(rodio::OutputStream, rodio::Sink)> {
    let mut stream_handler = rodio::OutputStreamBuilder::open_default_stream()?;
    stream_handler.log_on_drop(false);

    let sink = rodio::Sink::connect_new(stream_handler.mixer());

    let source = create_infinite_playback(ALARM_AUDIO_DATA)?;

    sink.append(source);
    sink.play();

    Ok((stream_handler, sink))
}

#[derive(Debug, Clone)]
pub struct Timer {
    created_timestamp: Instant,
    duration: Duration,
}

impl Timer {
    fn new(duration: Duration) -> Self {
        Self {
            created_timestamp: Instant::now(),
            duration,
        }
    }

    fn duration_to_string(duration: Duration) -> String {
        let mut string = String::with_capacity(8);

        let hour = duration.as_secs() / 3600;
        if hour > 0 {
            write!(string, "{:02}:", hour).unwrap();
        }

        write!(
            string,
            "{:02}:{:02}",
            duration.as_secs() / 60,
            duration.as_secs() % 60
        )
        .unwrap();

        if duration.is_zero() {
            write!(string, " (Type `timer stop` to stop the alarm!)").unwrap();
        }

        string
    }

    pub fn time_remaining_str(&self) -> String {
        Self::duration_to_string(self.time_remaining())
    }

    pub fn time_remaining(&self) -> Duration {
        self.duration
            .checked_sub(self.created_timestamp.elapsed())
            .unwrap_or(Duration::ZERO)
    }
}

impl FromStr for Timer {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(anyhow!("Timer string cannot be empty"));
        }

        let times = s.split(":").collect::<Vec<&str>>();

        if times.len() < 2 {
            return Err(anyhow!("Must include minutes and seconds"));
        }

        let seconds: u64;
        let minutes: u64;
        let mut hours = 0u64;

        match times.len() {
            2 => {
                minutes = times[0].parse()?;
                seconds = times[1].parse()?;
            }
            3 => {
                hours = times[0].parse()?;
                minutes = times[1].parse()?;
                seconds = times[2].parse()?;
            }
            _ => {
                return Err(anyhow!("Invalid format!"));
            }
        }

        if seconds >= 60 || minutes >= 60 {
            return Err(anyhow!("Invalid time!"));
        }
        if hours > 100 {
            return Err(anyhow!("Hours must be less than or equal to 100"));
        }

        let duration = Duration::from_secs(seconds + minutes * 60 + hours * 3600);

        if duration.is_zero() {
            return Err(anyhow!("The timer must not be zero seconds long!"));
        }

        Ok(Self::new(duration))
    }
}

#[cfg(test)]
mod tests {
    use crate::timer::Timer;
    use std::str::FromStr;
    use std::time::Duration;

    #[test]
    fn timer_from_str() {
        assert_eq!(
            Timer::from_str("1:00").unwrap().duration,
            Duration::from_secs(60)
        );

        assert_eq!(
            Timer::from_str("10:00").unwrap().duration,
            Duration::from_secs(600)
        );

        assert_eq!(
            Timer::from_str("2:30").unwrap().duration,
            Duration::from_secs(150)
        );
    }

    #[test]
    fn timer_invalid_str() {
        assert!(Timer::from_str("what").is_err());
        assert!(Timer::from_str("huh:10").is_err());
        assert!(Timer::from_str("sfjklwahl;fjwka;jkfdwadfsa:jwkla;f;jwda").is_err());
        assert!(Timer::from_str("13:000").is_ok());
        assert!(Timer::from_str("10:123").is_err());
        assert!(Timer::from_str("2000:00:00").is_err());
        assert!(Timer::from_str("0:0").is_err());
    }
}
