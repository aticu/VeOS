TARGET_FILES += $(TARGET_DIR)/bin/test
BUILD_DIRS += test/target
INITRAMFS_FILES += /bin/test
FMT_DIRS += test

$(TARGET_DIR)/bin/test: test/target/$(BUILD_TARGET)/$(BUILD_TYPE)/test
	@mkdir -p $(shell dirname $@)
	cp $< $@

test/target/$(BUILD_TARGET)/$(BUILD_TYPE)/test: test/target/$(BUILD_TARGET)/$(BUILD_TYPE)/libtest.a
	$(LINKER) $(LINKER_FLAGS) $< -o $@

test/target/$(BUILD_TARGET)/$(BUILD_TYPE)/libtest.a: $(shell find test/src -name "*.rs") test/Cargo.toml $(STD_FILES)
	cd test && $(RUST_COMPILER) build $(RUST_COMPILER_FLAGS)