RUST_LOG=risk5=trace

run-test:
	bzcat assets/addiw.json.log.bz2 | env RUST_LOG=$(RUST_LOG) cargo run

run:
	cargo run --bin risk5 2>&1 |tail -n 100

expand:
	cargo expand --bin u1

test:
	cargo test -- --nocapture --color=always --test-threads=1

clean:
	cargo clean
