RUST_LOG=risk5=warn

test: check unit-tests compliance-tests bbl-test

check:
	cargo check

unit-tests:
	cargo test -- --nocapture --color=always --test-threads=1

bbl-test: target/release/logrunner
	bzcat assets/bbl.bincode.bz2 |env STOP_AT=543900 ./target/release/logrunner

bbl-run: target/release/logrunner
	bzcat assets/bbl.bincode.bz2 |RUST_LOG=risk5=warn ./target/release/logrunner

COMPLIANCE_PATHS := $(wildcard compliance/tests/*.elf)
COMPLIANCE_TESTS := $(patsubst compliance/tests/%.elf,%-compliance-test,$(COMPLIANCE_PATHS))
COMPLIANCE_LOGS := $(patsubst compliance/tests/%.elf,compliance/logs/%.bincode.log.bz2,$(COMPLIANCE_PATHS))
.SECONDARY: $(COMPLIANCE_LOGS)

compliance-tests: $(COMPLIANCE_TESTS)

%-compliance-test: compliance/logs/%.bincode.log.bz2 target/release/logrunner
	bzcat $< |./target/release/logrunner

compliance/logs/%.bincode.log.bz2:
	rm -rf work-$*
	mkdir work-$*
	cd work-$* && \
		env LD_LIBRARY_PATH=$(shell pwd)/compliance/lib ../compliance/bin/spike ../compliance/tests/$*.elf && \
		cat log.json |../target/release/convert |bzip2 > ../compliance/logs/$*.bincode.log.bz2
	rm -rf work-$*

target/release/logrunner:
	cargo build --release

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

save-broken:
	git add Makefile Cargo.* bin src u1 bitcalc
	git commit -m'save broken'
	git push
	git st

load:
	git pull
	make test

convert: target/release/logrunner
	cat assets/bbl.json.log.bz2 |pv -cN read |bunzip2 |pv -cN uncomp |target/release/convert |pv -cN convert |bzip2 |pv -cN write > assets/bbl.bincode.bz2
