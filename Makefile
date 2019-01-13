RUST_LOG=risk5=warn

test: check unit-tests all-compliance-tests bbl-test

check:
	cargo check

unit-tests:
	cargo test -- --nocapture --color=always --test-threads=1

bbl-test: target/release/logrunner
	bzcat assets/bbl.json.log.bz2 |env STOP_AT=595283 ./target/release/logrunner

COMPLIANCE_PATHS := $(wildcard compliance/tests/*.elf)
COMPLIANCE_TESTS := $(patsubst compliance/tests/%.elf,%-compliance-test,$(COMPLIANCE_PATHS))
COMPLIANCE_LOGS := $(patsubst compliance/tests/%.elf,compliance/logs/%.json.log.bz2,$(COMPLIANCE_PATHS))
.SECONDARY: $(COMPLIANCE_LOGS)

all-compliance-tests: $(COMPLIANCE_TESTS)

%-compliance-test: compliance/logs/%.json.log.bz2 target/release/logrunner
	bzcat $< | env ./target/release/logrunner

compliance/logs/%.json.log.bz2:
	env LD_LIBRARY_PATH=$(shell pwd)/compliance/lib ./compliance/bin/spike compliance/tests/$*.elf
	bzip2 log.json
	mv log.json.bz2 compliance/logs/$*.json.log.bz2

target/release/logrunner:
	cargo build --bin logrunner --release

run:
	env RUST_LOG=$(RUST_LOG) cargo run --bin risk5 --release

run-dev:
	env RUST_BACKTRACE=1 RUST_LOG=$(RUST_LOG) cargo run --bin risk5

.PHONY: target/release/logrunner

clean:
	cargo clean

save: test
	git add Makefile Cargo.* bin src u1 bitcalc
	git commit -m'save'
	git push
	git st

load:
	git pull
	make test
