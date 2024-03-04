#!/usr/bin/env bash

set -euo pipefail

BACKEND_LIB_DIR=java/lib/src/main/resources
JNI_NAME=dsnp_graph_sdk_jni

for possible_library_name in "lib${JNI_NAME}.dylib" "lib${JNI_NAME}.so" "${JNI_NAME}.dll"; do

  for possible_target in "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu"; do
      possible_library_path="target/${possible_target}/release/${possible_library_name}"
      if [ -e "${possible_library_path}" ]; then
          DESTDIR="${BACKEND_LIB_DIR}/${possible_target}"
          echo "Installing ${possible_library_name} to $DESTDIR"
          install -d ${DESTDIR}
          install "${possible_library_path}" ${DESTDIR}
      fi
  done
done
