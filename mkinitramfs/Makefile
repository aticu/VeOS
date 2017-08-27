arch ?= x86_64

prog_name := mkinitramfs

target_dir := ../target

executable := target/release/$(prog_name)

rust_compiler_flags := --release
rust_compiler := cargo

.PHONY: all
all: $(executable)

.PHONY: clean
clean:
	rm -rf target

$(executable): cargo

.PHONY: cargo
cargo:
	$(rust_compiler) build $(rust_compiler_flags)

.PHONY: copy_to_target
copy_to_target: $(executable)

.PHONY: run
run: $(executable)
	$(executable) $(target_dir)/conf/mkinitramfs $(target_dir)/boot/initramfs $(target_dir)
