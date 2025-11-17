#!/usr/bin/env bash

set -eu

caffeinate -d -- cargo run --release

if [[ $? -eq 0 ]]; then
	say "Optimization succeeded"
	open plot.png
else
	say "Optimization succeeded"
fi

