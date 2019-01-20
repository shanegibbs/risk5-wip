SHELL=/bin/bash
PWD:=$(shell pwd)

RUST_LOG=risk5=warn

# release mode by default as runtime with debug mode
# is crazy slow
BUILD_MODE=release

ifeq ($(BUILD_MODE),release)
CARGO_BUILD_ARGS=build --release
else ifeq ($(BUILD_MODE),debug)
CARGO_BUILD_ARGS=build
else
$(error Bad BUILD_MODE value: $(BUILD_MODE))
endif

BUILD_DIR=$(PWD)/target/$(BUILD_MODE)
CONVERT=$(BUILD_DIR)/convert
LOGRUNNER=$(BUILD_DIR)/logrunner
RISK5=$(BUILD_DIR)/risk5

ASSETS:=$(PWD)/assets
SPIKE=$(PWD)/compliance/bin/spike
COMPLIANCE_PATHS := $(wildcard compliance/tests/*.elf)
COMPLIANCE_TESTS := $(patsubst compliance/tests/%.elf,%-compliance-test,$(COMPLIANCE_PATHS))
COMPLIANCE_LOGS := $(patsubst compliance/tests/%.elf,compliance/logs/%.bincode.log.bz2,$(COMPLIANCE_PATHS))

# keep intermediate files. Otherwise make delete
.SECONDARY: $(COMPLIANCE_LOGS)

# always run
.PHONY: build run test

# default target
test: check unit-tests compliance-tests bbl-test

check:
	cargo check

unit-tests:
	cargo test -- --nocapture --color=always --test-threads=1

bbl-test: build
	cat $(ASSETS)/bbl.bincode |env STOP_AT=694689 $(LOGRUNNER)

bbl-run: build
	cat $(ASSETS)/bbl.bincode |env RUST_LOG=risk5=warn $(LOGRUNNER)

build:
	cargo $(CARGO_BUILD_ARGS)

run: build
	env RUST_LOG=$(RUST_LOG) $(RISK5)

clean:
	cargo clean

save: test
	git add Makefile Cargo.* bin src u1 bitcalc
	git commit -m'save'
	git push
	git status

save-broken:
	git add Makefile Cargo.* bin src u1 bitcalc
	git commit -m'save broken'
	git push
	git status

load:
	git pull
	bunzip2 -k $(ASSETS)/bbl.bincode.bz2
	bunzip2 -k $(ASSETS)/bbl.bincode.bz2
	make test

compliance-tests: $(COMPLIANCE_TESTS)

%-compliance-test: compliance/logs/%.bincode.log.bz2 build
	bzcat $< |$(LOGRUNNER)

compliance/logs/%.bincode.log.bz2: build
	rm -rf work-$*
	mkdir work-$*
	cd work-$* && \
		env LD_LIBRARY_PATH=$(PWD)/compliance/lib \
			$(SPIKE) $(PWD)/compliance/tests/$*.elf && \
		cat log.json \
			|$(CONVERT) \
			|bzip2 \
			> $(PWD)/compliance/logs/$*.bincode.log.bz2
	rm -rf work-$*

# read bbl.log.jsonl compress to gz and bz2, convert
# to bincode and compress that output to gz and bz2 as well
# converts:
# - bbl.log.jsonl > bbl.log.jsonl.{gz,bz2}
# - bbl.log.jsonl > bbl.log.bincode
# - bbl.log.bincode > bbl.log.bincode.{gz,bz2}
import-log: build
	rm -rf $(ASSETS)/bbl.log.jsonl.{gz,bz2}
	rm -rf $(ASSETS)/bbl.log.bincode.{gz,bz2}
	rm -rf $(ASSETS)/bbl.log.bincode
	pv -rbpe $(ASSETS)/bbl.log.jsonl \
		|tee >(bzip2 >$(ASSETS)/bbl.log.jsonl.bz2) \
		|tee >(gzip -n >$(ASSETS)/bbl.log.jsonl.gz) \
		|$(CONVERT) \
		|tee >(bzip2 >$(ASSETS)/bbl.log.bincode.bz2) \
		|tee >(gzip -n >$(ASSETS)/bbl.log.bincode.gz) \
		> $(ASSETS)/bbl.log.bincode

# convert bbl.log.jsonl > bbl.log.bincode
convert-raw: build
	rm -rf $(ASSETS)/bbl.log.bincode
	pv -rbpe $(ASSETS)/bbl.log.jsonl \
		|$(CONVERT) \
		> $(ASSETS)/bbl.log.bincode

unpack-assets:
	pv -rvpe $(ASSETS)/bbl.log.jsonl.bz2 \
		|bunzip \
		|gzip -n > $(ASSETS)/bbl.log.jsonl.gz

perf: build
	bzcat $(ASSETS)/bbl.bincode.bz2 \
		|env RUST_LOG=risk5=warn STOP_AT=20000 
			valgrind \
				--tool=callgrind \
				--dump-instr=yes \
				--collect-jumps=yes \
				--simulate-cache=yes \
				$(LOGRUNNER)
