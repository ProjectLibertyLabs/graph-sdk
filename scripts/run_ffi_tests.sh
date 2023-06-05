#!/bin/bash

# Generate bindgen
make bindgen
cargo build --release -p dsnp-graph-sdk-ffi
# Copy library to c_example
cp target/release/libdsnp_graph_sdk_ffi.a bridge/ffi/src/c_example/

# Build main binary
cd bridge/ffi/src/c_example/
gcc main.c -L. -Wl,-rpath=. -ldsnp_graph_sdk_ffi -lsodium -lm -o main

# Run main binary
export LD_LIBRARY_PATH=bridge/ffi/src/c_example/
output=$(./main)
echo "$output"
echo "::set-output name=output::$output"

# Check test result
if [[ "${output}" == *"passed"* ]]; then
  echo "All tests passed!"
else
  echo "Some tests failed!"
  exit 1
fi
