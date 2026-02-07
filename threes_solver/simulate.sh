#!/usr/bin/env bash

set -eu

cargo run --release -- simulate "$@"

# 6144 with lookahead-depth 3
# cargo run --release -- --lookahead-depth 3 --seed 6273387017077892403 simulate "$@"
# but only 768 with lookahead-depth 2!
# cargo run --release -- --seed 6273387017077892403 simulate "$@"

# 3072
# cargo run --release -- --seed 291715960768821435 simulate "$@"
# cargo run --release -- --seed 16935663829556139934 simulate "$@"


