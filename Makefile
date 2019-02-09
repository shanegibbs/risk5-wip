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

export RUSTFLAGS=-C target-cpu=native

BUILD_DIR=$(PWD)/target/$(BUILD_MODE)
VALIDATE=$(BUILD_DIR)/validate
VALIDATE_STREAM=$(BUILD_DIR)/validate-stream
CONVERT=$(BUILD_DIR)/convert
LOGRUNNER=$(BUILD_DIR)/logrunner
FILTER=$(BUILD_DIR)/filter
RISK5=$(BUILD_DIR)/risk5

ASSETS:=$(PWD)/assets
COMPLIANCE_PATH=$(ASSETS)/compliance
TRANS_LOG_PATH=$(ASSETS)/transactions-logs
SPIKE_TRACE=env LD_LIBRARY_PATH=$(COMPLIANCE_PATH)/lib $(COMPLIANCE_PATH)/bin/spike
COMPLIANCE_PATHS := $(wildcard $(COMPLIANCE_PATH)/tests/*.elf)
COMPLIANCE_TESTS := $(patsubst $(COMPLIANCE_PATH)/tests/%.elf,%-compliance-test,$(COMPLIANCE_PATHS))
COMPLIANCE_LOGS := $(patsubst $(COMPLIANCE_PATH)/tests/%.elf,$(COMPLIANCE_PATH)/logs/%.bincode.log.bz2,$(COMPLIANCE_PATHS))

# keep intermediate files. Otherwise make delete
.SECONDARY: $(COMPLIANCE_LOGS)

# always run
.PHONY: build run test

# default target
test: check unit-tests compliance-tests spike-trace-test

test-failed:
	cat failed.bincode |RUST_LOG=risk5=trace cargo run --bin validate-single

check:
	cargo check

unit-tests:
	cargo test -- --nocapture --color=always --test-threads=1

spike-trace-test: build
	$(SPIKE_TRACE) --isa rv64ima -c2000000 $(ASSETS)/bbl |$(LOGRUNNER)

spike-trace: build
	$(SPIKE_TRACE) --isa rv64ima $(ASSETS)/bbl |$(LOGRUNNER)

spike-trace-trans: build
	$(SPIKE_TRACE) --isa rv64ima $(ASSETS)/bbl |$(VALIDATE)

build-logs: build
	$(SPIKE_TRACE) --isa rv64ima $(ASSETS)/bbl |pv -rbp |$(FILTER)

TRANS_LOG_PATHS := $(wildcard $(TRANS_LOG_PATH)/*.trans.log.gz)
TRANS_LOG_TESTS := $(patsubst $(TRANS_LOG_PATH)/%.trans.log.gz,%-trans-log-test,$(TRANS_LOG_PATHS))

trans-log-test: $(TRANS_LOG_TESTS)

%-trans-log-test: build
	zcat $(TRANS_LOG_PATH)/$*.trans.log.gz |$(VALIDATE_STREAM)

spike:
	env LD_LIBRARY_PATH=$(shell pwd)/assets/spike ./assets/spike/spike -d assets/bbl

build:
	cargo $(CARGO_BUILD_ARGS)

run: build
	env RUST_LOG=$(RUST_LOG) $(RISK5)

clean:
	cargo clean

save: test
	git add Makefile Cargo.* bin src
	git commit -m'save'
	git push
	git status

save-broken:
	git add Makefile Cargo.* bin src
	git commit -m'save broken'
	git push
	git status

load:
	git pull
	make test

compliance-tests: $(COMPLIANCE_TESTS)

%-compliance-test: build
	$(SPIKE_TRACE) --isa rv64ima $(COMPLIANCE_PATH)/tests/$*.elf |$(LOGRUNNER)

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
	$(SPIKE_TRACE) --isa rv64ima -c1000000 $(ASSETS)/bbl \
		|env RUST_LOG=risk5=warn valgrind \
			--tool=callgrind \
			--dump-instr=yes \
			--collect-jumps=yes \
			--simulate-cache=yes \
			$(VALIDATE)

perf-run: build
	env RUST_LOG=risk5=warn valgrind \
		--tool=callgrind \
		--dump-instr=yes \
		--collect-jumps=yes \
		--simulate-cache=yes \
		$(RISK5)
