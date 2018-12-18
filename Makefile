export RUST_LOG=risk5=trace

run:
	cargo run --bin risk5 2>&1 |tail -n 100

expand:
	cargo expand --bin u1

test:
	cargo test -- --nocapture --color=always --test-threads=1
