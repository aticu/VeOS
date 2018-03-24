MKINITRAMFS := mkinitramfs/target/release/mkinitramfs
BUILD_DIRS += mkinitramfs/target

$(TARGET_DIR)/boot/initramfs: $(MKINITRAMFS) $(TARGET_DIR)/conf/mkinitramfs $(patsubst %,$(TARGET_DIR)%,$(INITRAMFS_FILES))
	@mkdir -p $(shell dirname $@)
	$(MKINITRAMFS) $(TARGET_DIR)/conf/mkinitramfs $(TARGET_DIR)/boot/initramfs $(TARGET_DIR)

$(MKINITRAMFS): $(shell find mkinitramfs/src -name "*.rs")
	cd mkinitramfs && cargo build --release