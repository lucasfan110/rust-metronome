# Subdivision Setting

## Ideas

* Ok, so the user will put in a string of something like "x-x", where the amount of character is dependent on the subdivision. If the subdivision is 3, then the input string should have a length of 3  
* An 'x' represents to play the subdivided beat, while a '-' represents silence  
* If it's the downbeat of the first beat, then it's always played. So for example if the subdivision is 3 and subdivision setting is "--x", then the first beat of the downbeat is played even if the first beat is silence. But the rest of the downbeats are silent.  
* So should be straight forward enough

## Implementation

1. Ok... so some refactoring regarding to the logic that controls when the beat is played should be implemented...  
2. Create a struct in main called metronome beat tracker  
3. In the loop in main, ask the metronome beat tracker if it's time to play a beat  
4. If the function returned `Option::Some` with the beat info, then...  
   5. Based on the subdivision setting in metronome data, check if this beat should be played. If not, then continue the loop  
   6. Based on the beat info, which should contain information about the current beat and the current subdivided beat, play the metronome beat.

## Beat Info Struct

* The current beat, as u8  
* The current subdivided beat, as u8

### Next Subdivided Beat

1. A function that takes the number of beat and number of subdivided beat.  
2. Add 1 to the subdivided beat in self  
3. Modulo it by the number of subdivided beat  
4. If the subdivided beat in self is equal to 0, then  
   5. Add 1 to the current beat in self  
   6. Modulo it by the number of beat

## Metronome Beat Tracker Struct

* An `Arc` to `RwLock` to a `MetronomeData`  
* An instant, storing the timestamp of the last time the beat is played.

### Should Play Beat

1. A function that takes nothing  
2. Returns an optional beat info struct  
3. Check if the time elapsed since the last time the beat is played is greater than the duration per subdivided beat  
   4. If true, create a copy of the current beat info  
   5. Move the beat to the next subdivided beat  
   6. Set the timestamp of when the last time the beat is played to now  
   7. Return the copy of the current beat info  
8. Return none otherwise

## Metronome Setting Struct

* A vector of bools called play beat

### From String

1. A function that takes a string  
2. Returns a result of self or anyhow error  
3. Create a vector of bools  
4. Loop through each character in string.  
   5. If the character is not "x" or "-", then return an error  
   6. Push true to the vector of bools if character is equal to "x", otherwise false  
7. Return the self struct with the vector of bools

### Should Play Beat

1. A function that takes \&self and current subdivision beat  
2. Returns true or false, indicating if the current beat should be played  
3. Try to get the bool at play beat index of current subdivision beat  
4. If it's none, then default to true  
5. Return the result.

## Metronome Data

* Subdivision setting, which should be the metronome setting struct  
* Beat is replaced from u8 to the beat info struct