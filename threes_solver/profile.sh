#!/usr/bin/env bash

set -eu

cargo build --profile profiling

caffeinate -d -- samply record --output run_logs/profile.json.gz ../target/profiling/threes_solver --seed 0 --profiling "$@" 2>&1 | tee run_logs/profile.log

