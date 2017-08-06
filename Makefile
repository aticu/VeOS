arch ?= x86_64
target ?= $(arch)-unknown-none-gnu
build_type ?= debug

modules := kernel

target_dir := target

iso := image.iso

make_args := arch=$(arch) target=$(target) build_type=$(build_type)

.PHONY: all
all: copy_to_target

.PHONY: copy_to_target
copy_to_target:
	$(foreach module,$(modules),cd $(module) && $(MAKE) copy_to_target $(make_args))

.PHONY: clean
clean:
	$(foreach module,$(modules),cd $(module) && $(MAKE) clean $(make_args))
	rm -rf target $(iso)

.PHONY: run
run: $(iso)
	qemu-system-x86_64 -cdrom $(iso) --no-reboot -smp cores=4 -s

run_verbose: $(iso)
	qemu-system-x86_64 -cdrom $(iso) -d int --no-reboot -smp cores=4 -s

$(iso): copy_to_target
	grub-mkrescue -o $(iso) $(target_dir) 2>/dev/null

.PHONY: $(modules)
$(modules):
	cd $@ && $(MAKE) $(make_args)
