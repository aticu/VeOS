src_dir := src
build_dir := build

arch ?= x86_64
target := $(arch)-unknown-linux-gnu
kernel := $(build_dir)/kernel-$(arch).bin
iso := $(build_dir)/os-$(arch).iso
rust_lib := target/$(target)/debug/libVeOS.a

asm_folders := $(src_dir)/arch/$(arch)/init

assembly_source_files := $(foreach dir, $(asm_folders), $(wildcard $(dir)/*.asm))
assembly_object_files := $(patsubst $(src_dir)/%.asm, $(build_dir)/%.o, $(assembly_source_files))
grub_cfg := $(src_dir)/arch/$(arch)/grub.cfg


linker_script := $(src_dir)/arch/$(arch)/linker.ld
linker_flags := -n -T $(linker_script)
linker := ld
assembler_flags := -felf64
assembler := nasm

.PHONY: all clean run iso

all: $(kernel)

clean:
	rm -rf $(build_dir) target

run: $(iso)
	qemu-system-x86_64 -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	grub-mkrescue -o $(iso) build/isofiles 2>/dev/null

$(kernel): $(assembly_object_files) $(linker_script) $(rust_lib)
	$(linker) $(linker_flags) -o $(kernel) $(assembly_object_files) $(rust_lib)

$(assembly_object_files): $(build_dir)/%.o : $(src_dir)/%.asm
	@mkdir -p $(shell dirname $@)
	$(assembler) $(assembler_flags) $< -o $@

$(rust_lib):
	cargo build --target $(target)
