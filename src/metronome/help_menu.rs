pub fn print_help() {
    println!("Commands: ");
    println!("pause, p: Pause the metronome");
    println!("resume, r: Resume the metronome");
    println!("quit, q: Exit the metronome");
    println!("help, h: Print help");
    println!("clear, c: Clear the screen");
    println!("tempo, t <TEMPO>: Set the tempo of the metronome");
    println!("\tExample: t 60");
    println!("time <TIME_SIGNATURE>: Set the time signature of the metronome");
    println!("tempo-type, tt <TEMPO_TYPE>: Set the tempo type of the metronome");
    println!(
        "\tExample: `tt dotted-quarter` changes the current tempo type \
        from whatever to dotted quarter note equals"
    );
    println!(
        "subdivision, s <SUBDIVISION>: Set the subdivision of the metronome. \
        Type `s` to clear subdivision"
    );
    println!(
        "subdivision-setting, ss <SUBDIVISION_SETTING>: Set which subdivided beat \
        to play. \"x\" represents play and \"-\" represents silent."
    );
    println!(
        "\tExample: `ss -x` with a subdivision of 2 will only play downbeat. \
        Type `ss` to clear subdivision setting."
    );
    println!(
        "\tYou can set subdivision setting \
        if there are no subdivision and the time signature is eights, to change its \
        subdivision"
    );
    println!(
        "timer <TIME> | stop: Set a timer, with the format of `HH:MM:SS`, \
        where hours and minutes are optional. Use `timer stop` to stop the timer."
    );
    println!(
        "tap: Enters tap mode. Press return for each beat, and after 4 taps, the \
        tempo of the metronome will automatically change to the tapped tempo. Press \
        `q` to stop"
    )
}
