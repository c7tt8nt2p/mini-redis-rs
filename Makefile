test-unit:
	cargo test -p client -p server

test-integration:
	cargo test

test-all:
	cargo test --workspace

test-all-coverage-html:
	# cargo +stable install cargo-llvm-cov --locked
	cargo llvm-cov --workspace --html --open