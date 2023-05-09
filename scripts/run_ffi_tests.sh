#!/bin/bash

# Generate bindgen
make bindgen

# Copy library to c_example
cp target/release/libdsnp_graph_sdk_ffi.a bridge/ffi/src/c_example/

# Build main binary
cd bridge/ffi/src/c_example/
gcc main.c -L. -ldsnp_graph_sdk_ffi -lm -o main

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
