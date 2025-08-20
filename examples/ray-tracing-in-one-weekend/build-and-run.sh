#!/bin/bash

cargo run --bin neco-felis-compile -- ./main.fe -o a.out --ptx
./a.out > image.pnm
pnm2png image.pnm image.png
