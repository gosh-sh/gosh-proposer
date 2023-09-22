
.PHONY: install-dev
install-dev:
	cargo install --profile dev --path .

.PHONY: install
install:
	cargo install --profile release --path .

.PHONY: fmt
fmt:
	taplo fmt
	cargo +nightly fmt --all -v

.PHONY: fix
fix:
	cargo clippy --fix --allow-dirty

.PHONY: run
run:
	cargo run --release

.PHONY: check
check:
	cargo check --release

.PHONY: debug_run
debug_run:
	GOSH_LOG=trace cargo run --releaes

.PHONY: test
test:
	cargo test --release
