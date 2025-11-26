## Version 0.3.3 (11/25/2025)

* Minor code refactor
* Updated beat2 audio so that there isn't a slight silence at the start of the audio

## Version 0.3.2 (11/21/2025)

* Updated clap to 4.5.53
* Quality of life change, now if the metronome is paused or in tap mode, entering other commands or pressing enter won't make the message disappear, so the user will always know if they are in tap mode or they are paused without confusion.

## Version 0.3.1 (11/16/2025)

* Updated clap to 4.5.51
* Statically link the audio file so no additional installation is required. Can use right out of the box

## Version 0.3.0 (11/14/2025)

* Added subdivision
* Ability to set subdivision within the metronome while it's running
* Minor code stylistic changes

## Version 0.2.2 (11/11/2025)

* Changed from using `Mutex` to `RwLock`
* Minor code optimizations

## Version 0.2.1 (11/08/2025)

* Minor fix when resuming, starting on beat 1 instead of the last beat.

## Version 0.2.0 (11/08/2025)

-   Ability to accept user input while in the metronome
-   Can change the tempo, the time signature, the tempo type, and other while in
    the metronome without needing to restart
-   Added tap mode, to let the user tap out the tempo while in the metronome.

## Version 0.1.0 (11/02/2025)

-   Basic functions of a metronome
-   Enter the metronome with a set tempo, a time signature, and a tempo type
-   Supports wide variety of time signatures, like `6/8`, `3/4`, `2/2`, etc.
-   When in time signatures like `6/8`, automatically enters a subdivide mode.
