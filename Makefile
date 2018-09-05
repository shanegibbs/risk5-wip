export RUST_LOG=risk5=trace

run: test
	cargo run

test:
	cargo test -- --nocapture --color=always --test-threads=1
