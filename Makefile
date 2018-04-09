include config.mk

ifeq ($(BUILD_TYPE),release)
	RUST_COMPILER_FLAGS += --release
endif

export RUST_TARGET_PATH=$(PWD)/targets

TARGET_FILES := $(TARGET_DIR)/conf/mkinitramfs $(TARGET_DIR)/boot/initramfs
BUILD_DIRS := $(TARGET_DIR)
INITRAMFS_FILES :=

.PHONY: all
all: target_files

include $(patsubst %,%/module.mk,$(MODULES))

.PHONY: target_files
target_files: $(TARGET_FILES)

.PHONY: clean
clean:
	rm -rf $(BUILD_DIRS)

.PHONY: run
run: $(ISO)
	qemu-system-x86_64 -cdrom $(ISO) $(QEMU_FLAGS) -enable-kvm

.PHONY: run_debug
run_debug: $(ISO)
	qemu-system-x86_64 -cdrom $(ISO) $(QEMU_FLAGS) -d int -S

.PHONY: gdb
gdb:
	gdb $(KERNEL_BINARY) -ex "target remote :1234"

$(ISO): all
	grub-mkrescue -o $(ISO) $(TARGET_DIR) 2>/dev/null

$(TARGET_DIR)/conf/mkinitramfs:
	@mkdir -p $(shell dirname $@)
	echo $(INITRAMFS_FILES) | tr " " "\n" > $@