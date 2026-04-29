# SPDX-License-Identifier: GPL-2.0
# kconfirm makefile

TARGET := kconfirm

HAS_CARGO := $(shell command -v cargo 2> /dev/null)

# Extra arguments forwarded to kconfirm.
# Example: make kconfirm KCONFIRM_ARGS="--enable dead_links"
KCONFIRM_ARGS ?=

PHONY += all
all: $(TARGET)

$(TARGET):
ifdef HAS_CARGO
	@cargo run --release -p kconfirm-linux -- --linux-path ../../ $(KCONFIRM_ARGS)
else
	@echo "Error: Cargo not found. Please install Rust and Cargo to build kconfirm."
	@false
endif

clean:
ifdef HAS_CARGO
	@cargo clean > /dev/null 2>&1
else
	@:
endif
