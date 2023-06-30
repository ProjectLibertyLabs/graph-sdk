#!/bin/bash

# Run main binary
output=$(bridge/ffi/src/c_example/main)
echo "$output"
echo "::set-output name=output::$output"

# Check test result
if [[ "${output}" == *"passed"* ]]; then
  echo "All tests passed!"
else
  echo "Some tests failed!"
  exit 1
fi
