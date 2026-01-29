# Accented Beat

* So... I'm thinking that in the metronome data there should be another field called `accented_beats` which is a vector of metronome beat accent enum marking which beat is accented.

* Metronome beat accent is an enum containing three variants: accented, beat, and subdivision.
  
  * It should be a C-style enum. It doesn't really matter too much but by being a C-style enum it can help with accessing some array elements and stuff

* A distinguished function that takes a time signature and returns the corresponding accented beats. Make it a function so it can handle all the weird scenarios like `7/8` and `5/8`, which I need for fucking practicing some passages.

* So a standardized way of marking accents that both the sound module and the UI module can use.
