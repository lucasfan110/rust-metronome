# Timer

1. Should be a new command
2. Used like so `timer HH:MM:SS`. If only two fields are supplied (e.g. `12:00`), then default to `MM:SS`. Same if there is only one field supplied
3. Also another command which is `stop`, and it stops the timer.

## Thoughts

* Hmm... Right now the metronome data stores the timer object which contains a created time stamp and the duration it should last

* And the audio is stored in the main function.

* So the audio and the actual timer is detached, and it's causing a bit of a problem.

* Maybe just like set the timer audio to none if a new timer starts too

## Execute

1. In `data.rs`, in `execute`, when the timer command is sent, parse the time string into a timer struct.
2. Set the timer in self to the parsed timer struct
3. If the command is `stop`, then set the timer struct to none in metronome data.

## Main

1. Create a timer render tracker, passing in the metronome data
2. If the timer render tracker deems it time to re-render the UI, then render the UI
3. If the duration left in the timer struct is zero, then play the alarm sound

## Timer Render Tracker

* Metronome data
* Last re-render timestamp

### New

1. Takes an arc to metronome data
2. Save it to self, with the metronome data being the metronome data, and the last re-render timestamp set as now

### Should Render Timer

1. If the timer struct in metronome data is none, then return false
2. Otherwise, return if the elapsed time since last re-render timestamp is greater than one second

### Just Rendered

1. Set the last re-render timestamp to now.

## Ui

### Render

1. Write the timer info to the string buffer in self

### Write Timer Info

1. If the timer field in metronome data is none, then return
2. Write out the time left

## Metronome Data

* Timer (Optional timer struct)

## Create Infinite Playback

1. Takes a reference to an `&[u8]` slice, which is the audio data
2. Try creating a new sink with the stream handle
3. Create a new decoder, passing in a cursor which reads the `&[u8]` slice.
4. Create a source that is loop
5. Return the sink and the source as a tuple

## Timer

* Created timestamp (Instant)
* Duration (Duration)
* A sink, to manage audio playback
* A stream handler

### New

1. Takes a duration
2. Returns self, with the timestamp being now, and the duration being the duration passed in.

### From Str

1. If the string is empty, then return an error
2. Split the string with the ":" char, and reverse it, collect it into a vector of strings
3. Convert the first element to an u64. If failed or if it's greater than 59, then return an error
4. Get the second str in the vector optionally, and map the string to an u64 that is not greater than 59, save it to a variable called mins
5. Same for the third str in the vector, save it to hours
6. Return Self::new, passing in Duration::from_secs(secs + mins * 60 + hours * 3600)

### Duration to String (Static)

1. Takes a duration, returns a string
2. Create a string with a capacity of 10
3. Create a variable called hour, set it to the duration as secs / 3600
4. If the hour is greater than 0, then write `{hour:02}:` into the string
5. Write minute, which is the secs in duration / 60, and the secs, which is the secs % 60, into the string.
6. Return the string

### Time Remaining

1. Create a variable called duration remaining, set it to `self.duration - self.timestamp.elapsed()`
2. Convert the duration into string.

### Play Audio

1. Create an infinite playback of the alarm audio
2. Save the sink to self
3. Play the audio.