#!/usr/bin/env bash
set -e
set -u
set -o pipefail

OUTFILE=run_logs/sts.log

while true; do
	cargo run --release -- --lookahead-depth 3 simulate > "$OUTFILE" 2>/dev/null
	# ./simulate.sh > "$OUTFILE" 2>/dev/null
	if grep -q "11m6144" "$OUTFILE" ; then
		grep seed "$OUTFILE"
		break
	fi
	echo -n "."
done

