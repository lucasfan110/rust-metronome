use std::io::Cursor;

use super::data::beat::accent::MetronomeBeatAccent;

static METRONOME_SOUNDS: &[&[u8]] = &[
    include_bytes!("../audio/beat1.mp3"),
    include_bytes!("../audio/beat2.mp3"),
    include_bytes!("../audio/beat3.mp3"),
];

pub fn play_metronome_sound(
    metronome_beat_accent: MetronomeBeatAccent,
) -> anyhow::Result<(rodio::OutputStream, rodio::Sink)> {
    let mut stream_handler = rodio::OutputStreamBuilder::open_default_stream()?;
    stream_handler.log_on_drop(false);

    let sink = rodio::play(
        stream_handler.mixer(),
        Cursor::new(METRONOME_SOUNDS[metronome_beat_accent as usize]),
    )?;

    Ok((stream_handler, sink))
}
