.DEFAULT_GOAL := all

PROFILE := release

# Determine the operating system
UNAME := $(shell uname)

PROTOC := protoc
 ifeq ($(UNAME), Darwin)
	PROTOC = /opt/homebrew/opt/protobuf@21/bin/protoc
endif

CBINDGEN=${HOME}/.cargo/bin/cbindgen

.PHONY: check
check:
	@echo "Running Cargo check..."
	@cargo check --all --all-features --all-targets

.PHONY: test
test:
	@echo "Running Cargo test..."
	@cargo test --all --all-targets

.PHONY: clippy
clippy:
	@echo "Running Cargo clippy..."
	@cargo clippy --all --all-features --all-targets -- -D warnings

.PHONY: deny
deny:
	@echo "Running Cargo deny..."
	@cargo deny check -c .cargo-deny.toml

.PHONY: format
format:
	@echo "Running Cargo fmt..."
	@cargo fmt --all
format-check:
	@echo "Running Cargo fmt..."
	@cargo fmt --all -- --check

.PHONY: build
build:
	@echo "Running Cargo build..."
	@cargo build --all --all-features --all-targets

.PHONY: build-node
build-node:
	@echo "Build Neon Node Bridge for GraphSDK..."
	@cd bridge/node && npm install && npm run native:build-release && npx jest --verbose


.PHONY: dsnp-graph-sdk-jni
	@cargo build -p dsnp-graph-sdk-jni --profile $(PROFILE)

.PHONY: doc
doc:
	@echo "Running Cargo doc..."
	@RUSTDOCFLAGS="--enable-index-page --check -Zunstable-options" cargo doc --no-deps --all-features

.PHONY: clean
clean:
	@echo "Running Cargo clean..."
	@cargo clean

.PHONY: capacities
capacities:
	@echo "Generating graph page capacities..."
	@cargo test --features=calculate-page-capacity calculate_page_capacities; cargo fmt

.PHONY: all
all: check test clippy deny format build doc

.PHONY: ci-local
ci-local: all

$(CBINDGEN):
	cargo install cbindgen

.PHONY: bindgen
bindgen: $(CBINDGEN)
	@echo "Running bindgen..."
	@( cd ./bridge/ffi && $(CBINDGEN) --config cbindgen.toml --crate dsnp-graph-sdk-ffi --output ./src/c_example/dsnp_graph_sdk_ffi.h )

.PHONY: clean-ffi-bridge
clean-ffi-bridge:
	cargo clean -p dsnp-graph-sdk-ffi

.PHONY: build-ffi-bridge-for-test
build-ffi-bridge-for-test:
	@echo "Building FFI for tests..."
	cargo build --profile $(PROFILE) -p dsnp-graph-sdk-ffi

.PHONY: build-ffi-tests
build-ffi-tests: build-ffi-bridge-for-test bindgen
	$(MAKE) -C bridge/ffi/src/c_example clean all

.PHONY: test-ffi
test-ffi: build-ffi-tests
	@echo "Running FFI tests..."
	@scripts/run_ffi_tests.sh

.PHONY: test-jni
test-jni: build-jni
	@( cd java ; ./gradlew test --rerun-tasks)

test-java-client:
	@( cd java/example-graphsdk-client ; ./gradlew test --rerun-tasks)

test-node-client:
	@( cd bridge/node/node-example-client ; npm run test )

.PHONY: build-jni
build-jni:
	@echo "Build JNI ..."
	cargo build -p dsnp-graph-sdk-jni --profile $(PROFILE)
	@./scripts/install_jni.sh

.PHONY: install-protobuf-codegen
install-protobuf-codegen:
	@cargo install protobuf-codegen ; PATH="${HOME}/.cargo/bin:${PATH}"

.PHONY: install-protos
ifeq ($(UNAME), Darwin)
install-protos: install-protobuf-codegen
	@echo "Installing protobuf package..."
	# Latest version of protobuf (@23) has flagged Rust codegen as experimental;
	# we'll stick with an earlier version (@21) until that's resolved.
	@brew install protobuf@21
endif
ifeq ($(UNAME), Linux)
install-protos: install-protobuf-codegen
	$(error "Please update the Makefile with the appropriate commands for installing the protobuf package using the appropriate distro package manager")
endif

.PHONY: build-protos
build-protos: build-rust-protos build-java-protos

.PHONY: build-rust-protos
build-rust-protos:
	@echo "Generating Rust protobuf types..."
	@rm -f ./bridge/common/src/proto_types/*
	@$(PROTOC) --rust_out ./bridge/common/src/proto_types ./bridge/common/protos/input.proto ./bridge/common/protos/output.proto

.PHONY: build-java-protos
build-java-protos:
	@echo "Generating Java protobuf types..."
	@rm -f ./java/lib/src/main/java/io/amplica/graphsdk/models/*
	@$(PROTOC) --java_out ./java/lib/src/main/java/ ./bridge/common/protos/input.proto ./bridge/common/protos/output.proto
