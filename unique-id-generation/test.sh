#!/usr/bin/env bash
cargo build --release
../maelstrom/maelstrom test -w unique-ids --bin target/release/echo --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
