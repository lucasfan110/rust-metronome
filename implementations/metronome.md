# Metronome

1. Parse the command line argument
2. Define a `TICK_LENGTH` constant at the top, which is a `Duration` type of `1` microsecond.
3. Define a `UI_TICK_LENGTH` constant at the top, which is a Duration type with 10 milliseconds.
4. Create a variable called `metronome_data`, which will be an `Arc<Mutex<MetronomeData>>`. It acts as the data center for the program, where the UI, the metronome, and the user input will interact with. Initialize it with the data parsed from command line arguments
5. Create a variable called `last_beat_time`, and assign it to `Instant::now() - duration_per_beat`
6. Create a variable called `last_ui_render_time`, and assign it `Instant::now() - UI_TICK_LENGTH`
7. Create a variable called `ui`, and initialize it.
8. Create a variable called `metronome_sound`, and assign it `MetronomeSound::new()`
9. Create an mpsc channel, with type of the user input enum
10. Spawn a new thread, with move closure
    1. Here, enter an infinite loop
    2. Use stdin to wait for a user input.
    3. Convert the user input string into a user input enum
    4. Send it using the mpsc channel

11. Enter an infinite loop
    1. Sleep for `TICK_LENGTH` duration.
    2. If `Instant::now() - last_beat_time` is greater than `metronome_data.duration_per_beat`.
       1. Set `last_beat_time` equals to `Instant::now()`.
       2. Set `metronome_data.beat` equal to `metronome_data.beat + 1 % time_signature.0`
       3. if `time_signature.1` is equal to 8
          1. Play accented sound if `beat` equals to 0. If beat is divisible by 3, then play normal beat sound. Otherwise, play subdivision sound
       4. Otherwise, Play the metronome sound type accented if `metronome_data.beat` equals to `0`. Otherwise, play metronome sound type beat.
       5. Call `ui.render`
    3. Try to receive a message from the mpsc channel
       1. If received a message, then
       2. If the message is to pause, then set is paused in metronome data to true
       3. If the message is to resume, then set is paused in metronome data to false, and reset the beat to time_signature.0 - 1.
       4. If the message is to quit, then exit the program.


## Structs

### Command Line Arguments

* Tempo `u16` (Required)
  * The tempo of the metronome in beats per second. By default it's 1 beat per quarter note
* Tempo type `Option<TempoType>`
  * The note type 1 beat is equal to. By default is `QuarterNote`.
* Time signature `String` (Optional but positional)
  * The time signature in the format of `NUMERATOR/DENOMINATOR`, like `4/4` or `6/8` or `2/2`
  * The numerator can be any non-zero number. The denominator must be an exponent of 2 (e.g. 1, 2, 4, 8, 16)
  * Default is `4/4`
  * Check the format of time signature, stop the program from starting if it's not in a fraction format or the numbers are not a whole number.
* Subdivision `u8`
  * The subdivision as a number.
  * Example, if subdivision is given `2`, then one beat is divided into two.
  * Should not be 0. Default is `1`
* Subdivision setting `String`
  * A string that customizes the subdivision, with greater control for the user to control what beat the subdivided beat should sound.
  * For example, if the user puts `--subdivision 3`, and then put `--subdivision-setting 001`, the `001` is the setting. It should be a string with the same number of digits as the subdivision. A `0` represents that the subdivided beat is silent. A `1` represents that the subdivided beat is played, and a `-` represents a slur.
    * Another example is `--subdivision 4 --subdivision-setting 01-1`. This represents a silent first beat of subdivision, the 2nd and 3rd beat of the subdivision slurred together, and the 4th beat of the subdivision. When slurred together, the metronome beat will not elongate. So the above setting is the same as `--subdivision-setting 0101`. 

### Metronome Data

* Tempo `u16`
* Secs per beat `Duration`
  * Dependent on tempo. Should never be set alone. Can only be changed through changing tempo
* Beats played `i32`
* Tempo type `TempoType`
* Time signature `TimeSignature`
* Subdivision `u8`
* Subdivision setting `TODO!!!`
* Is paused `boolean`

#### New

1. A function that takes a Cli reference
2. `tempo = cli.tempo`
3. `time_signature = cli.time_signature.into()`
4. `duration_per_beat = get_duration_per_beat()`
5. `tempo_type = cli.tempo_type.unwrap_or`
   1. Check if the denominator is equal to 8. If it is, then set the tempo type to dotted quarter note equals.
   2. If the denominator is equal to 2, then set the tempo type to half note equals
   3. Otherwise, default to quarter note equals
6. `subdivision = cli.subdivision`

#### Set Tempo

1. A function that takes `&mut self`, and a `u16` tempo
2. Set `self.tempo` to tempo
3. Set `self.duration_per_beat` to `get_duration_per_beat()`
4. Set beats played to 0


### Metronome Sound

* Stream Handle
  * Whatever the type is for the stream handler in rodio
* Sink (Optional)
  * Whatever returns from `rodio::play`

#### Play

1. A function that takes `&mut self` an argument `metronome_type`, which is the `MetronomeSoundType`.
2. Get the file name based on `metronome_type`, which should be a path of `audio/beat{METRONOME_TYPE}.mp3`.
3. Open the file, and save it to a variable called `beat_audio`.
4. Play the `beat_audio` using `rodio::play`, and save the return value of `rodio::play` into `self.sink`.

### Time Signature

1. A tuple struct with `(u8, u8)`

#### From Str

1. Implement the trait `From<&str>` for `TimeSignature`
2. Should already be valid so no need to error check
3. Split the str from the `/`
4. Convert the first one to an `u8`, called `numerator`
5. Convert the second one to an `u8`, called `denominator`
6. Return the struct `TimeSignature(numerator, denominator)`

#### Display

1. Implement `fmt::Display` for Time Signature
2. Write `{}/{}, self.0, self.1`

### UI

* Metronome data `Arc<Mutex<MetronomeData>>`
* Stdout `Stdout`

#### Render

1. A function that takes `&mut self`
2. Returns `std::io::Result<()>`
3. Clear the terminal
4. Move the cursor to `(0, 0)`
5. Print out information about the tempo, about the time signature, the subdivision, and the subdivision setting
6. Print out the metronome beat

#### Print out info

1. Takes `&mut self`
2. Print out the tempo in the format of `Tempo: {TEMPO}`, and then followed by 3 tabs, and then the time signature, in the format of `Time Signature: {TIME_SIGNATURE}`, followed by 3 tabs. Lastly, print out the subdivision in the format of `Subdivision: {SUBDIVISION}`. If subdivision is 1, then print out `None`

#### Print out metronome beat

1. Takes `&mut self`
2. The function should print out the metronome beat, in the format of `[	X	x	x	x	]`, where a capital X represents the start of the measure and the rest are the beats. It depends on the time signature, and the current beat should be highlighted as an italic blue text.
3. Print out a `[\t`
4. Loop `i` through `0..time_signature.0`
   1. Create a variable called `character_to_print`. If `i` is 0, then it's `X` (uppercase), otherwise it's `x` (lowercase)
   2. If `i` is equal to beats_played, then print out `character_to_print`, italicized and also in blue
   3. Otherwise, just print out `character_to_print`
   4. Print out a tab character
5. Finish with a print of the `]` bracket.

## Enums

### Tempo Type

* Quarter note
  * The default. One beat equals quarter note
* Eighth note
* Sixteenth note
* Half note
* Whole note
* Dotted quarter
* Dotted half
* Dotted whole

#### To Note Length

1. A function that takes `self`
2. Returns the note length of the tempo type
3. Quarter note = 1
4. Eighth note = 0.5
5. Sixteenth note = 0.25
6. Half note = 2
7. Whole note = 1
8. Dotted quarter = 1.5
9. Dotted half = 3
10. Dotted whole = 4

### Metronome Sound Type

1. Accented = 1
2. Beat = 2
3. Subdivision = 3

### User Input

* Pause
* Resume
* Quit

## Functions

### Get Duration Per Beat 

1. A function that takes a tempo as `u16`, a tempo type, and time signature
2. Create a variable called `note_length` and it'll be `tempo_type.to_note_length()`.
3. Create a variable called `new_tempo` and it's equal to tempo. It should be a `f64`
4. `new_tempo` is equal to `new_tempo` times `time_signature.1 / (4.0 / note_length)`
5. Returns `Duration::from(60.0 / new_tempo)`

## Other Features

* A tempo tapping mode, where the program enters a state that keeps on asking user to press `Enter`, and depending on the tempo that the user is pressing `Enter` at, return the averaged tempo
* Ability to save metronome settings and load it.
