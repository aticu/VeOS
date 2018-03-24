TARGET_FILES += $(TARGET_DIR)/bin/init
BUILD_DIRS += init/target
INITRAMFS_FILES += /bin/init

$(TARGET_DIR)/bin/init: init/target/$(BUILD_TARGET)/$(BUILD_TYPE)/init
	@mkdir -p $(shell dirname $@)
	cp $< $@

init/target/$(BUILD_TARGET)/$(BUILD_TYPE)/init: init/target/$(BUILD_TARGET)/$(BUILD_TYPE)/libinit.a
	$(LINKER) $(LINKER_FLAGS) $< -o $@

init/target/$(BUILD_TARGET)/$(BUILD_TYPE)/libinit.a: $(shell find init/src -name "*.rs")
	cd init && $(RUST_COMPILER) build $(RUST_COMPILER_FLAGS)