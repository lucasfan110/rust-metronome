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

pub struct Timer {
    created_timestamp: Instant,
    duration: Duration,
    sink: rodio::Sink,
    _stream_handler: rodio::OutputStream,
}

impl Timer {
    fn new(duration: Duration) -> Result<Self, rodio::StreamError> {
        let mut stream_handler = rodio::OutputStreamBuilder::open_default_stream()?;
        stream_handler.log_on_drop(false);

        Ok(Self {
            created_timestamp: Instant::now(),
            duration,
            sink: rodio::Sink::connect_new(stream_handler.mixer()),
            _stream_handler: stream_handler,
        })
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

    pub fn play_alarm(&self) -> anyhow::Result<()> {
        let source = create_infinite_playback(ALARM_AUDIO_DATA)?;

        self.sink.append(source);
        self.sink.play();

        Ok(())
    }
}

impl FromStr for Timer {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(anyhow!("Timer string cannot be empty"));
        }

        let mut times = s.split(':').rev();

        let mut get_next_time = || times.next().map(u64::from_str).transpose();

        let secs = get_next_time()?.unwrap_or(0);
        let mins = get_next_time()?.unwrap_or(0);
        let hours = get_next_time()?.unwrap_or(0);

        let duration = Duration::from_secs(secs + mins * 60 + hours * 3600);

        Ok(Self::new(duration)?)
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
    #[should_panic]
    fn timer_invalid_str() {
        Timer::from_str("what").unwrap();
        Timer::from_str("huh:10").unwrap();
    }
}
