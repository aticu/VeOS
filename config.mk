ARCH ?= x86_64
BUILD_TYPE ?= debug
BUILD_TARGET := $(ARCH)-unknown-veos-gnu

MODULES := kernel init test mkinitramfs

TARGET_DIR := target

ISO := image.iso

RUST_COMPILER_FLAGS := --target $(BUILD_TARGET)
RUST_COMPILER := xargo

LINKER := ld
LINKER_FLAGS := --gc-sections

QEMU_FLAGS := --no-reboot -smp cores=4 -s -serial stdio