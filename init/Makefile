arch ?= x86_64
target ?= $(arch)-unknown-veos-gnu
build_type ?= debug

linker_flags := --gc-sections
linker := ld

prog_name := init

target_dir := ../target

rust_lib := target/$(target)/$(build_type)/lib$(prog_name).a

executable := target/$(target)/$(build_type)/$(prog_name)

rust_compiler_flags := --target $(target)
ifeq ($(build_type),release)
	rust_compiler_flags += --release
endif
rust_compiler := xargo

.PHONY: all
all: $(executable)

.PHONY: clean
clean:
	rm -rf target

$(executable): cargo
	$(linker) $(linker_flags) $(rust_lib) -o $@

.PHONY: cargo
cargo:
	echo $(RUST_TARGET_PATH)
	$(rust_compiler) build $(rust_compiler_flags)

.PHONY: copy_to_target
copy_to_target: $(executable)
	@mkdir -p $(target_dir)/bin
	cp $(executable) $(target_dir)/bin/$(prog_name)
