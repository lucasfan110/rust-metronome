# Tap Mode

1. In the execute function for the enum user input, if the message is tap, then enters tap mode
2. Set the tap mode in the metronome data to true
3. Now, go to main
4. Create a variable called tempo measurer
5. In the input thread, check if the tap mode in the metronome data is true. If it is, then
   1. Do not convert the string to message and send it
   2. Trim the string
   3. Check if the string is equal to `quit` or `q`. If it is, then set the tap mode in the metronome data to false.
      1. Clear the tempo measurer.
   4. Otherwise, call the tap method for tempo measurer
   5. If the number of times already tapped is at least 4, then
      1. Call the calculate tempo method on the tempo measurer
      2. Send a message of set tempo, with the estimated tempo calculated.

## Enums

### User Input

* Add a tap mode user input
  * Will be sent when the user types `tap` while in the metronome

## Structs

### Tempo Measurer

* Has a vector of instants

#### Tap

* Push Instant::now into the vector of instants

#### Calculate Tempo

* The vector should have at least two elements
* Fold the vector, with an initial value of a tuple, with the first element being the total duration, and the second being the last instant.
  * For each instant, return a tuple, with the first value being the first value of the accumulator plus instant minus the second element of the accumulator, and the second value being the instant.
  * Should get a tuple finally, with the first value being the total duration
* Divide the total duration by the length of the vector minus 1. That's the duration per beat
* Returns the inverse of the duration per beat multiplied by 60, as u16.

#### Num Tapped

* Returns the length of the vector

#### Clear

* Clears the vector