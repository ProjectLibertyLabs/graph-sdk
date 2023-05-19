#!/bin/bash

set -euo pipefail

BACKEND_LIB_DIR=java/lib/src/main/resources
JNI_NAME=dsnp_graph_sdk_jni

cargo build -p dsnp-graph-sdk-jni --release

for possible_library_name in "lib${JNI_NAME}.dylib" "lib${JNI_NAME}.so" "${JNI_NAME}.dll"; do
  possible_library_path="target/release/${possible_library_name}"
  echo "$possible_library_path"
  if [ -e "${possible_library_path}" ]; then
    cp "${possible_library_path}" "${BACKEND_LIB_DIR}/"
    break
  fi
done
