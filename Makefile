RUST_LOG=risk5=warn

test: check unit-tests addiw-test
	bzcat assets/bbl.json.log.bz2 |env STOP_AT=595283 RUST_LOG=risk5=error cargo run --release --bin logrunner

check:
	cargo check

bbl-test:
	bzcat assets/bbl.json.log.bz2 | env RUST_LOG=$(RUST_LOG) cargo run --release --bin logrunner

addiw-test:
	bzcat assets/addiw.json.log.bz2 | env RUST_LOG=risk5=error cargo run --release --bin logrunner

run:
	env RUST_LOG=$(RUST_LOG) cargo run --bin risk5 --release

run-dev:
	env RUST_BACKTRACE=1 RUST_LOG=$(RUST_LOG) cargo run --bin risk5

unit-tests:
	cargo test -- --nocapture --color=always --test-threads=1

clean:
	cargo clean

save: test
	git add Makefile Cargo.* bin src u1
	git commit -m'save'
	git push
	git st

load:
	git pull
