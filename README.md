# termcalc
A terminal based calculator written in Rust. Port of https://github.com/svenslaggare/termcalc.

## Usage
Works like a normal calculator. The calculator can be in the following modes:
* float: arithmetic done in 64-bits floating point (default).
* int: arithmetic done in 64-bits integer.

Each mode has different predefined functions and constants.

### Defining variables and functions
Variables can be defined as: `x = 4`. Functions are defined as: `f(x)=x^2`.

### Predefined
Variables and functions can be predefined by adding a file `default.termcalc` in your home folder.

## Build
* Requires cargo (https://rustup.rs/).
* Build with: `cargo build --release`
* Build output in `target/release/termcalc`
