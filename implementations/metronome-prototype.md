# Metronome Prototype

A command line interface called `metro`

1. Takes a tempo at least, in beats per minutes, used like this `metro 60`, where the tempo is quarter notes = 60 beats per minutes
2. Play a metronome beat in `4/4` time signature by default, where the first beat is accented.
3. Print out some text, where first line is the information about the current metronome session, printing out information about the current tempo, the current time signature, and the current subdivision.
4. Repeat until the user presses control C, where the program will just end.

## Command Line Options

* **Time signature**: the time signature, in the format of `{NUMBER}/{NUMBER}`. Like `6/8` or `3/4` or `2/2`
* **Tempo**: Indicates what a beat is equal to. If used like `metro 60 -t 8th`, then based on the time signature, each beat's speed would be adjusted.
  * Valid values:
  * `16th`: Sixteenth note = tempo
  * `8th`: Eighth note = tempo
  * `4th`: Quarter note = tempo
  * `2nd`: Half note = tempo
  * `whole`: Whole note = tempo
  * `dotted-quarter` or `dq`: Dotted quarter note = tempo (Useful for time signatures like `3/8` or `6/8`)
  * `dotted-half`: Dotted half note = tempo
  * `dotted-whole`: Dotted whole note = tempo

## Play metronome beat in 4/4 (basic)

1. For now, the simplest, assume time signature is `4/4`. 
2. Load the metronomes beats. For now there is only one, so use that one. 
3. Go into an infinite loop
4. Play the metronome beat
5. Sleep for $\frac{1}{tempo / 60}$ amount of seconds. Make sure it's in `f64`.

## Play metronome beat in 4/4 with accents

1. The first note should be accented. It should play `beat1.mp3`. The rest should play `beat2.mp3`
2. Well... Add a variable outside the loop called `beats_played`. Should be a number initialized to 0
3. If `beats_played` is 0, then play `beat1.mp3`
4. Otherwise, play `beat2.mp3`.
5. Every time a beat is played, add `1` to `beats_played`, and then it should be moduloed by the amount of beats given, like the numerator in the time signature.

## Ability to pause

1. In another thread, there should be an stdin waiting to read user's input.
2. If the user inputs a letter `p`, then 

## Print out UI

1. Based on the `beats_played` variable, print out something like `[X    x    x    .]`, where X represents an accented beat, lowercase x represents an unaccented beat, and a `.` represents beat that haven't been played yet.
2. I can use a vector with size of the number of beats, called `ui_print`. Set `ui_print[beats_played]` to `x`, and uppercase if it's 0
3. Reset the `ui_print` to all `.` when `beats_played` is 0.
4. Print `ui_print`, joined by a `\t`, surrounded by a bracket for now.
5. Reprint on the same line every time