test-unit:
	cargo test -p client -p server

test-integration:
	cargo test

test-all:
	cargo test --workspace