.DEFAULT_GOAL := all

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

.PHONY: dsnp-graph-sdk-jni
	@cargo build -p dsnp-graph-sdk-jni --release

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
all: check test clippy format build doc

.PHONY: ci-local
ci-local: all

.PHONY: bindgen
bindgen:
	@echo "Running bindgen..."
	@./scripts/run_bindgen.sh

.PHONY: test-ffi
test-ffi:
	@echo "Running FFI tests..."
	@./scripts/run_ffi_tests.sh

.PHONY: test-jni
test-jni: build-jni
	@( cd java ; ./gradlew test --rerun-tasks)

.PHONY: build-jni
build-jni:
	@echo "Build JNI ..."
	cargo build -p dsnp-graph-sdk-jni --release
	@./scripts/install_jni.sh

install-protos:
	@echo "Install PROTO ..."
	@brew install protobuf; cargo install protobuf-codegen; PATH="$HOME/.cargo/bin:$PATH"

build-protos:
	@echo "Build PROTO ..."
	@./scripts/build_protos.sh
