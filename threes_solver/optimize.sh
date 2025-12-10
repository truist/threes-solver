#!/usr/bin/env bash

set -eu

# caffeinate -d -- cargo run --release -- optimize "$@"

# DYLD_INSERT_LIBRARIES=/Users/truist/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/librustc-nightly_rt.asan.dylib \
# RUSTFLAGS="-Z sanitizer=address -C debuginfo=2" \
# cargo +nightly run --release -- optimize "$@"

RUSTFLAGS="-C debuginfo=2" cargo build --release

MallocNanoZone=0 \
MallocScribble=1 \
MallocGuardEdges=1 \
lldb ../target/release/threes_solver -- --seed 3696195633902677747 optimize
# MallocCheckHeapStart=1 \
# MallocCheckHeapEach=1 \

if [[ $? -eq 0 ]]; then
	say "Optimization succeeded"
	open plot.png
else
	say "Optimization succeeded"
fi

