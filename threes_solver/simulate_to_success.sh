#!/usr/bin/env bash
set -e
set -u
set -o pipefail

OUTFILE=run_logs/sts.log

while true; do
	./simulate.sh > "$OUTFILE" 2>/dev/null
	if grep -q 3072 "$OUTFILE" ; then
		grep seed "$OUTFILE"
		break
	fi
	echo -n "."
done

