Rust puzzle solver (WORK IN PROGRESS)
=====================================

**CAUTION this is a work in progress !**

[![Build Status](https://travis-ci.org/svalat/rust-puzzle-solver.svg?branch=master)](https://travis-ci.org/svalat/rust-puzzle-solver)

This project is a fun project I made to learn Rust. The idea is to solve a puzzle by taking the pieces in photo on a white paper.

The tool take out the pieces and try to match them all together to form the puzzle then provide the final solution.

In order to proceed for big puzzle we need to take multiple picture to have enought resolution on all pieces for precision.


How to build
------------

You first need to install Rust, look on the dedicated website : https://www.rust-lang.org/.

You the just need to run cargo :

```sh
cargo build --release
```

You can then run on one image :

```sh
./target/release/rust-puzzle-solver tests/real-1.png
```

If you add option `--dump` will output pictures into the current directory with solution and steps of the solver to help debugging.
You can also control the number of threads with `--threads`.

Licence
-------

This project is distributed under CeCILL-C licence equivalent and compatible with LGPL.
