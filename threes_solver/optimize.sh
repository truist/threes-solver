#!/usr/bin/env bash

set -eu

caffeinate -d -- cargo run --release

if [[ $? -eq 0 ]]; then
	say "Profiling succeeded"
	open plot.png
else
	say "Profiling succeeded"
fi

