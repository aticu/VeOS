arch ?= x86_64
build_type ?= debug

modules := kernel init test

target_dir := target

iso := image.iso

make_args := arch=$(arch) build_type=$(build_type)

initramfs := $(target_dir)/boot/initramfs

.PHONY: all
all: copy_to_target $(initramfs)

.PHONY: copy_to_target
copy_to_target:
	$(foreach module,$(modules),$(MAKE) -C $(module) copy_to_target $(make_args) &&) true
	cp -r conf $(target_dir)

.PHONY: clean
clean:
	$(foreach module,$(modules),$(MAKE) -C $(module) clean $(make_args) && ) true
	rm -rf target $(iso)
	$(MAKE) -C mkinitramfs clean $(make_args)

.PHONY: run
run: $(iso)
	qemu-system-x86_64 -cdrom $(iso) --no-reboot -smp cores=4 -s -enable-kvm

run_verbose: $(iso)
	qemu-system-x86_64 -cdrom $(iso) -d int --no-reboot -smp cores=4 -s

$(iso): all
	grub-mkrescue -o $(iso) $(target_dir) 2>/dev/null

.PHONY: $(modules)
$(modules):
	$(MAKE) -C $@ $(make_args)

.PHONY: initramfs
initramfs:
	$(MAKE) -C mkinitramfs run $(make_args)

$(initramfs): initramfs
