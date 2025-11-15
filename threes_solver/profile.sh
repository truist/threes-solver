#!/usr/bin/env bash

set -eu

cargo build --profile profiling

caffeinate -d -- samply record ../target/profiling/threes_solver --seed 0 --profiling

