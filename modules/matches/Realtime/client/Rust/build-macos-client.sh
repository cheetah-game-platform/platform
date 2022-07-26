#!/bin/bash

## сборка relay клиента для тестирования на macos
OUTPUT="../Unity/games.cheetah.matches.realtime/"

## macos
rm -f $OUTPUT/x86_64/libcheetah_matches_realtime_client.bundle
cargo build
cp ../../../../target/debug/libcheetah_matches_realtime_client.dylib "$OUTPUT/Runtime/Library/macos.bundle"