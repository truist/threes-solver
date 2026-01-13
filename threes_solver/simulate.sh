#!/usr/bin/env bash

set -eu

cargo run --release -- --seed 291715960768821435 simulate "$@"

