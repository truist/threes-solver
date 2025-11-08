#!/usr/bin/env bash

set -eu

cargo build --profile profiling

samply record ../target/profiling/threes_solver

