RUST_LOG=risk5=trace

test: unit-tests addiw-test
	bzcat assets/bbl.json.log.bz2 |head -n 17987 | RUST_LOG=risk5=error cargo run

bbl-test:
	bzcat assets/bbl.json.log.bz2 | env RUST_LOG=$(RUST_LOG) cargo run

addiw-test:
	bzcat assets/addiw.json.log.bz2 | env RUST_LOG=risk5=error cargo run

run:
	cargo run --bin risk5 2>&1 |tail -n 100

unit-tests:
	cargo test -- --nocapture --color=always --test-threads=1

clean:
	cargo clean

save: test
	git add Makefile Cargo.* src u1
	git commit -m'save'
	git push
	git st

load:
	git pull
