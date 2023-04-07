#!/usr/bin/env bash
cargo build --release
../maelstrom/maelstrom test -w echo --bin target/release/echo --node-count 1 --time-limit 10
