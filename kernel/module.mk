TARGET_FILES += $(TARGET_DIR)/boot/grub/grub.cfg $(TARGET_DIR)/boot/kernel.bin
BUILD_DIRS += kernel/target

LINKER_SCRIPT := kernel/src/arch/$(ARCH)/linker.ld

KERNEL_BUILD_TARGET := $(ARCH)-unknown-none-gnu
KERNEL_LINKER_FLAGS := -n -T $(LINKER_SCRIPT) --gc-sections

KERNEL_LIB := kernel/target/$(KERNEL_BUILD_TARGET)/$(BUILD_TYPE)/libveos.a

KERNEL_RUST_COMPILER_FLAGS := --target $(KERNEL_BUILD_TARGET)
ifeq ($(BUILD_TYPE),release)
	KERNEL_RUST_COMPILER_FLAGS += --release
endif

ASM_FOLDERS := kernel/src/arch/$(ARCH)/init
ASSEMBLY_SOURCE_FILES := $(foreach DIR, $(ASM_FOLDERS), $(wildcard $(DIR)/*.asm))
ASSEMBLY_OBJECT_FILES := $(patsubst kernel/src/%.asm,kernel/target/$(KERNEL_BUILD_TARGET)/build/%.o, $(ASSEMBLY_SOURCE_FILES))
ASSEMBLER := nasm
ASSEMBLER_FLAGS := -felf64

$(TARGET_DIR)/boot/grub/grub.cfg: kernel/src/arch/$(ARCH)/grub.cfg
	@mkdir -p $(shell dirname $@)
	cp $< $@

$(TARGET_DIR)/boot/kernel.bin: kernel/target/$(KERNEL_BUILD_TARGET)/build/kernel-$(ARCH).bin
	@mkdir -p $(shell dirname $@)
	cp $< $@

kernel/target/$(KERNEL_BUILD_TARGET)/build/kernel-$(ARCH).bin: $(ASSEMBLY_OBJECT_FILES) $(KERNEL_LINKER_SCRIPT) $(KERNEL_LIB)
	$(LINKER) $(KERNEL_LINKER_FLAGS) -o $@ $(ASSEMBLY_OBJECT_FILES) $(KERNEL_LIB)

$(KERNEL_LIB): $(shell find kernel/src -name "*.rs")
	cd kernel && $(RUST_COMPILER) build $(KERNEL_RUST_COMPILER_FLAGS)

$(ASSEMBLY_OBJECT_FILES): kernel/target/$(KERNEL_BUILD_TARGET)/build/%.o : kernel/src/%.asm
	@mkdir -p $(shell dirname $@)
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) $< -o $@