#!/usr/bin/env bash
set -e
set -u
set -o pipefail

caffeinate -d -w $$ &

for depth in 1 2 3 4; do
	for breadth in "--single-insertion-point" " "; do
		cargo run --release -- --seed 0 $breadth --lookahead-depth $depth simulate --batch
		echo ""
	done
done

