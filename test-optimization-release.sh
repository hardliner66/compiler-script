#!/usr/bin/env bash

cargo build --release
echo "Running unoptimized version..."
time bash -c "./target/release/compiler-script examples/c-like.parse.rn -i examples/demo.c-like | ./target/release/compiler-script examples/c-like.run.rn > /dev/null"

echo "Running optimized version..."
time bash -c "./target/release/compiler-script examples/c-like.parse.rn -i examples/demo.c-like | ./target/release/compiler-script examples/c-like.optimize.rn | ./target/release/compiler-script examples/c-like.run.rn > /dev/null"