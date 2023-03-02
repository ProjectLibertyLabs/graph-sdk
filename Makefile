.PHONY: check
check:
	@echo "Running Cargo check..."
	@cargo check --all --all-features --all-targets

.PHONY: test
test:
	@echo "Running Cargo test..."
	@cargo test --all --all-features --all-targets

.PHONY: clippy
clippy:
	@echo "Running Cargo clippy..."
	@cargo clippy --all --all-features --all-targets -- -D warnings

.PHONY: fmt
fmt:
	@echo "Running Cargo fmt..."
	@cargo fmt --all -- --check

.PHONY: audit
audit:
	@echo "Running Cargo audit..."
	@cargo audit

.PHONY: build
build:
	@echo "Running Cargo build..."
	@cargo build --all --all-features --all-targets

.PHONY: doc
doc:
	@echo "Running Cargo doc..."
	@cargo doc --all --all-features --all-targets

.PHONY: clean
clean:
	@echo "Running Cargo clean..."
	@cargo clean

.PHONY: all
all: check test clippy fmt audit build doc
