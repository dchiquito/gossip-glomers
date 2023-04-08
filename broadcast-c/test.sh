#!/usr/bin/env bash
cargo build --release
../maelstrom/maelstrom test -w broadcast --bin target/release/echo --node-count 5 --time-limit 20 --rate 10 --nemesis partition
