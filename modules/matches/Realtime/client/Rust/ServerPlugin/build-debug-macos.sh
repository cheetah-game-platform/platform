#!/bin/bash
## сборка runtime клиента для тестирования на macos
OUTPUT="../../Unity/games.cheetah.matches.realtime.plugin/"
cargo build
cp ../../../../../target/debug/libcheetah_matches_realtime_plugin.dylib "$OUTPUT/Runtime/FFI/Library/libcheetah_matches_realtime_plugin.bundle"
