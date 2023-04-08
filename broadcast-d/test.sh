#!/usr/bin/env bash
cargo build --release
../maelstrom/maelstrom test -w broadcast --bin target/release/echo --node-count 25 --time-limit 20 --rate 100 --latency 100
