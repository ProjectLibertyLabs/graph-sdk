#!/bin/bash

echo "Generating Rust code..."
rm -f ./bridge/common/src/proto_types/*
protoc --rust_out ./bridge/common/src/proto_types ./bridge/common/protos/input.proto ./bridge/common/protos/output.proto;

echo "Generating Java code..."
rm -f ./java/lib/src/main/java/io/amplica/graphsdk/models/*
protoc --java_out ./java/lib/src/main/java/ ./bridge/common/protos/input.proto ./bridge/common/protos/output.proto;
