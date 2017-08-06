arch ?= x86_64
target ?= $(arch)-unknown-none-gnu
build_type ?= debug

src_dir := src
build_dir := target/$(target)/build

target_dir := ../target

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

.PHONY: all
all: $(kernel)

.PHONY: clean
clean:
	rm -rf target

.PHONY: run
run: $(iso)
	qemu-system-x86_64 -cdrom $(iso) --no-reboot -smp cores=4 -s

.PHONY: debug
debug: $(iso)
	qemu-system-x86_64 -cdrom $(iso) -d int --no-reboot -smp cores=4 -s -S

.PHONY: run_verbose
run_verbose: $(iso)
	qemu-system-x86_64 -cdrom $(iso) -d int --no-reboot -smp cores=4 -s

.PHONY: iso
iso: $(iso)

.PHONY: doc
doc:
	cargo rustdoc -- --no-defaults --passes collapse-docs --passes unindent-comments --passes strip-priv-imports

.PHONY: doc_open
doc_open: doc
	xdg-open target/doc/veos/index.html

.PHONY: fmt
fmt:
	cargo-fmt -- --write-mode=overwrite

.PHONY: fmt-diff
fmt-diff:
	cargo-fmt -- --write-mode=diff

.PHONY: doctest
doctest:
	cargo rustdoc -- --no-defaults --passes collapse-docs --passes unindent-comments --passes strip-priv-imports --test

.PHONY: test
test: doctest
	RUSTFLAGS+=" -A dead_code" cargo test

.PHONY: objdump
objdump: $(kernel)
	objdump $(kernel) -D -C --disassembler-options=intel-mnemonic | less

.PHONY: hexdump
hexdump: $(kernel)
	hexdump -C -v $(kernel) | less

.PHONY: copy_to_target
copy_to_target: $(kernel) $(grub_cfg)
	@mkdir -p $(target_dir)/boot/grub
	@cp $(kernel) $(target_dir)/boot/kernel.bin
	@cp $(grub_cfg) $(target_dir)/boot/grub

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

.PHONY: cargo
cargo:
	$(rust_compiler) build $(rust_compiler_flags)
