
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

.PHONY: run_proposer
run_proposer:
	cargo run -p gosh_proposer --release

.PHONY: run_deposit
run_deposit:
	cargo run -p deposit-proposal-checker --release

.PHONY: run_withdraw
run_withdraw:
	cargo run -p withdraw_proposal_checker --release

.PHONY: check
check:
	cargo check --release

.PHONY: debug_run
debug_run:
	GOSH_LOG=trace cargo run -p gosh_proposer --releaes

.PHONY: test
test:
	cargo test -p gosh_proposer --release
