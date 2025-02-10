An extremely incomplete
[uxn](https://wiki.xxiivv.com/site/uxn.html)/[varvara](https://wiki.xxiivv.com/site/varvara.html)
environment.

## Usage

Run a ROM with:

```
cargo run whatever.rom
```

or run without an argument to start a minimal test program.

## Requirements

Tested on macOS, but should work anywhere [minifb](https://docs.rs/minifb/latest/minifb/) does.

To run tests, or the built-in program, `uxnasm` must be on your PATH. I'm using a copy built from
the [original 100 Rabbits implementation](https://sr.ht/~rabbits/uxn/).
