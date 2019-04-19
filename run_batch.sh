#!/bin/bash

node --experimental-modules process_batch.mjs $1/raw.txt $1/input.txt $1/std.txt
cargo run --release --  -o $1/output.txt -q $1/input.txt
vimdiff $1/output.txt $1/std.txt
