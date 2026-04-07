#!/usr/bin/env bash

cargo build
echo "Running unoptimized version..."
time bash -c "./target/debug/compiler-script examples/c-like.parse.rn -i examples/demo.c-like | ./target/debug/compiler-script examples/c-like.run.rn > /dev/null"

echo "Running optimized version..."
time bash -c "./target/debug/compiler-script examples/c-like.parse.rn -i examples/demo.c-like | ./target/debug/compiler-script examples/c-like.optimize.rn | ./target/debug/compiler-script examples/c-like.run.rn > /dev/null"