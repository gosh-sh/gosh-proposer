.PHONY: fmt
fmt:
	taplo fmt
	cargo +nightly fmt --all -v

.PHONY: fix
fix:
	cargo clippy --fix --allow-dirty

.PHONY: run_proposer
run_proposer:
	cargo run -p gosh_proposer --release

.PHONY: run_deposit
run_deposit:
	cargo run -p deposit-proposal-checker --release

.PHONY: run_withdraw
run_withdraw:
	cargo run -p withdraw_proposal_checker --release

.PHONY: get_blocks
get_blocks:
	cargo run -p withdraw_proposal_checker --release  -- get_last_blocks

.PHONY: check
check:
	cargo check --release

.PHONY: debug_run
debug_run:
	GOSH_LOG=trace cargo run -p gosh_proposer --releaes

.PHONY: test
test:
	cd tests && python test_all.py 2>&1 | tee test.log

.PHONY: build
build:
	cargo build --release

.PHONY: install
install: build
	cp target/release/gosh_proposer ~/.cargo/bin/
	cp target/release/deposit-proposal-checker ~/.cargo/bin/
	cp target/release/withdraw_proposal_checker ~/.cargo/bin/
	cp target/release/l2-telemetry ~/.cargo/bin/

