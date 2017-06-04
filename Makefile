arch ?= x86_64
target ?= $(arch)-unknown-none-gnu
build_type ?= debug

src_dir := src
build_dir := target/$(target)/build

kernel := $(build_dir)/kernel-$(arch).bin
iso := $(build_dir)/os-$(arch).iso
rust_lib := target/$(target)/$(build_type)/libveos.a

asm_folders := $(src_dir)/arch/$(arch)/init

assembly_source_files := $(foreach dir, $(asm_folders), $(wildcard $(dir)/*.asm))
assembly_object_files := $(patsubst $(src_dir)/%.asm, $(build_dir)/%.o, $(assembly_source_files))
grub_cfg := $(src_dir)/arch/$(arch)/grub.cfg


linker_script := $(src_dir)/arch/$(arch)/linker.ld
linker_flags := -n -T $(linker_script) --gc-sections
linker := ld
assembler_flags := -felf64
assembler := nasm
rust_compiler_flags := --target $(target)
ifeq ($(build_type),release)
	rust_compiler_flags += --release
endif
rust_compiler := xargo

.PHONY: all clean run iso run_verbose objdump cargo doc doc_open doctest test debug

all: $(kernel)

clean:
	rm -rf target

run: $(iso)
	qemu-system-x86_64 -cdrom $(iso) --no-reboot -s

debug: $(iso)
	qemu-system-x86_64 -cdrom $(iso) -d int --no-reboot -s -S

run_verbose: $(iso)
	qemu-system-x86_64 -cdrom $(iso) -d int --no-reboot -s

iso: $(iso)

doc:
	cargo rustdoc -- --no-defaults --passes collapse-docs --passes unindent-comments --passes strip-priv-imports

doc_open: doc
	xdg-open target/doc/veos/index.html

doctest:
	cargo rustdoc -- --no-defaults --passes collapse-docs --passes unindent-comments --passes strip-priv-imports --test

test: doctest
	RUSTFLAGS+=" -A dead_code" cargo test

objdump: $(kernel)
	objdump $(kernel) -D -C --disassembler-options=intel-mnemonic | less

hexdump: $(kernel)
	hexdump $(kernel) | less

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p $(build_dir)/isofiles/boot/grub
	@cp $(kernel) $(build_dir)/isofiles/boot/kernel.bin
	@cp $(grub_cfg) $(build_dir)/isofiles/boot/grub
	grub-mkrescue -o $(iso) $(build_dir)/isofiles 2>/dev/null

$(kernel): $(assembly_object_files) $(linker_script) cargo $(rust_lib)
	$(linker) $(linker_flags) -o $(kernel) $(assembly_object_files) $(rust_lib)

$(assembly_object_files): $(build_dir)/%.o : $(src_dir)/%.asm
	@mkdir -p $(shell dirname $@)
	$(assembler) $(assembler_flags) $< -o $@

cargo:
	$(rust_compiler) build $(rust_compiler_flags)
