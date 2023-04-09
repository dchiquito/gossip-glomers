#!/usr/bin/env bash
cargo build --release
../maelstrom/maelstrom test -w kafka --bin target/release/echo --node-count 1 --concurrency 2n --time-limit 20 --rate 1000
