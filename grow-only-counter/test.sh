#!/usr/bin/env bash
cargo build --release
# ../maelstrom/maelstrom test -w g-counter --bin target/release/echo --node-count 3 --time-limit 20 --rate 100 --nemesis partition
../maelstrom/maelstrom test -w g-counter --bin target/release/echo --node-count 1 --time-limit 5 --rate 10
