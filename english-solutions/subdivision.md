# Subdivision

## Thoughts

1. So... how should I do this  
2. I think first of all depending on the subdivision, I just need to divide the duration per beat by the amount of subdivision, and that'll be the duration per subdivision.  
3. Right now in the main thread loop, it'll sleep for a tick amount of time. And it'll check the timestamp of when the last beat is played, and see if the time elapsed.  
4. Ok, so I think I can add a subdivision beat to the metronome data structure, which will record the subdivided beat, which is also 0 indexed.  
5. Now back in the main thread loop... Since a subdivision beat only plays after a main beat is played, I've gotta do something here...  
6. I think I'll use an if-else loop. If the main `if should_play_beat` loop is true, then do the whole normal thing. But attach an else loop, where if it's time to play the subdivided beat, then play... a subdivided beat  
7. So I'll need a variable that tracks when the subdivided beat is played, and also probably another duration in metronome data to track how long per subdivided beat, so that I don't have to do `duration_per_beat` divided by the amount subdivided every time, which can be a little bit slower.  
8. Ok. That's it. I think I've got it. Now lemme see how to implement it

## Implementation

1. Go to the main function. Create a new variable underneath `last_beat_time` called `last_subdivided_beat_time`, and it should be set to `Instant::now()`.  
2. In the main thread loop, in the `if should_play_beat(...)` branch, after everything is done, make sure to reassign the timestamp for the last time the subdivided beat is played to `Instant::now` too.  
3. Add an else if branch after the `if should_play_beat(...)`  
4. The else if branch should check if it is time to play the subdivided beat, so use a helper function with that  
   5. In the body, play the subdivided metronome beat  
   6. Set the last subdivided beat timestamp to `Instant::now()`  
7. **Everything should work...?**

## Functions

### Should Play Subdivided Beat

1. A function that takes a reference to the metronome data and the timestamp of when the last subdivided beat is played  
2. Returns a `boolean`, determining if it's time to play a subdivided beat.  
3. Check if the metronome is paused with `metronome_data.is_paused`  
4. AND, check if the time elapsed since the last subdivided beat timestamp is greater than the duration per subdivided beat in the metronome data.

## Structs

### Metronome Data

* Duration per subdivided beat  
  * A duration

#### New

1. Set duration per subdivided beat to duration per beat divided by subdivision count.

#### Get Duration per Subdivided Beat

1. Returns the duration per subdivided beat

#### Set Subdivision

1. Make sure the subdivision is always greater than zero by using max  
2. Change the duration per subdivided beat, recalculate it by doing duration per beat divided by the subdivision count  
3. Recalculate duration per beat.

#### Recalculate Duration per Beat

1. Recalculate the duration per subdivided beat too, which is just the duration per beat divided by the subdivided amount.