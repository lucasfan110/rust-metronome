use UserInput::*;
use std::str::FromStr;

pub enum UserInput {
    Pause,
    Resume,
    Quit,
    Help,
    Clear,
    Tap,
    SetTempo(String),
    SetTempoDirect(u16),
    SetTimeSignature(String),
    SetTempoType(String),
    SetSubdivision(String),
    Unknown(String),
}

impl FromStr for UserInput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowercase = s.to_lowercase();
        let args: Vec<&str> = lowercase.split(" ").collect();

        if args.is_empty() {
            return Ok(Self::Unknown(String::new()));
        }

        let get_nth_arg =
            |index: usize| -> String { args.get(index).copied().unwrap_or_default().to_string() };

        Ok(match args[0] {
            "pause" | "p" => Pause,
            "resume" | "r" => Resume,
            "quit" | "q" => Quit,
            "help" | "h" => Help,
            "clear" | "c" => Clear,
            "tempo" | "t" => SetTempo(get_nth_arg(1)),
            "time" => SetTimeSignature(get_nth_arg(1)),
            "tempo-type" | "tt" => SetTempoType(get_nth_arg(1)),
            "subdivision" | "s" => SetSubdivision(get_nth_arg(1)),
            "tap" => Tap,
            command => Unknown(command.to_string()),
        })
    }
}
