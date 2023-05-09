#!/bin/bash

# Generate bindgen
cargo install cbindgen
cd ./bridge/ffi && 	cbindgen -v --config cbindgen.toml --crate dsnp-graph-sdk-ffi --output ./src/c_example/dsnp_graph_sdk_ffi.h