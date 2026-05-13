# SPDX-License-Identifier: GPL-2.0
# kconfirm makefile

TARGET := kconfirm

# Extra arguments forwarded to kconfirm.
# Example: make kconfirm KCONFIRM_ARGS="--enable-check dead_links"
KCONFIRM_ARGS ?=

$(TARGET):
	$(CARGO) run --release --offline -p kconfirm-linux -- --linux-path $(srctree) --enable-arch $(SRCARCH) $(KCONFIRM_ARGS)


clean-files += target vendor
