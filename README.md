# seedcracker_leaves

Crack a 32-bit minecraft seed knowing the oak tree placements
within a chunk, with their respective trunk height & leaves

Original code found at
https://github.com/19MisterX98/32BitSeedTreeReverseMC18

Video explanation (jump to around 15:57)
https://youtube.com/watch?v=gSxcDYCK_lY&t=15m57s

## Motivation

1. To do *anything* multithreaded, to show myself that
just because it has threads, doesnt mean I need to feel I'm
incapable of doing it. This is probably the simplest multi-
threaded scenario, where I just print the output 😄
2. There was a CTF for finding the minecraft seed, and this was
an approach I thought might work. I did want an improvement in
speed tho because I was tired of waiting 20 minutes with the Java
code.

(Unfortunately, I wasn't able to get the seed. It's possible
it wasn't a 32-bit seed to begin with, which wouldn't be surprising
since the footage felt rushed and last-minute)

## Time

I ran it against the Java code, and the Rust re-write is
about 2.5-ish times faster than the Java equivalent, when using
the same amount of threads. On my hardware, that means it went
from 22m 28s down to 8m 54s. Nice!

## Running

1. `git clone https://github.com/BareTuna/seedcracker_leaves.git`
2. `cd seedcracker_leaves`
3. `cargo run --release`

To substitute your own tree data, go to src/main.rs and you
should be able to find `let trees = trees![`. The comments in
there should help you figure it out
