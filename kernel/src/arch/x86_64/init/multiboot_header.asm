section .multiboot_header

extern LOADER_START
extern _bss_start
extern _kernel_end
extern start

;support for multiboot2
%define MULTIBOOT2_MAGIC 0xe85250d6
%define MULTIBOOT2_ARCHITECTURE 0x0
%define MULTIBOOT2_ADDRESS_TAG 2
%define MULTIBOOT2_ENTRY_ADDRESS_TAG 3
%define MULTIBOOT2_OPTIONAL_TAG 0
%define MULTIBOOT2_END_TAG 0

multiboot2_start:   ;start of multiboot2 header
    dd MULTIBOOT2_MAGIC
    dd MULTIBOOT2_ARCHITECTURE  ;protected mode i386
    dd multiboot2_end - multiboot2_start   ;length of header
    dd 0x100000000 -(MULTIBOOT2_MAGIC + MULTIBOOT2_ARCHITECTURE + (multiboot2_end - multiboot2_start))  ;checksum

    ;optional tags here
address_tag_start:
    dw MULTIBOOT2_ADDRESS_TAG
    dw MULTIBOOT2_OPTIONAL_TAG
    dd address_tag_end - address_tag_start   ;the size of this tag
    dq multiboot2_start                      ;the address of the beginning of the header
    dq LOADER_START                          ;the address at which the text segment should be loaded
    dq _bss_start                            ;the address at which the data segment ends
    dq _kernel_end                           ;the address at which the bss segment ends
address_tag_end:

entry_tag_start:
    dw MULTIBOOT2_ENTRY_ADDRESS_TAG
    dw MULTIBOOT2_OPTIONAL_TAG
    dd entry_tag_end - entry_tag_start       ;the size of this tag
    dq start                                 ;the entry address for the kernel
entry_tag_end:

    ;end tag
end_tag_start:
    dw MULTIBOOT2_END_TAG
    dw MULTIBOOT2_OPTIONAL_TAG
    dd end_tag_end - end_tag_start
end_tag_end:
multiboot2_end:     ;end of multiboot header

;support for multiboot
%define MULTIBOOT_MAGIC 0x1badb002
%define MULTIBOOT_FLAGS 0x10003

multiboot_start:    ;start of multiboot header
    dd MULTIBOOT_MAGIC
    dd MULTIBOOT_FLAGS
    dd -(MULTIBOOT_MAGIC + MULTIBOOT_FLAGS)  ;checksum
    dd multiboot_start                       ;physical address of the header
    dd LOADER_START                          ;the address at which loading should start
    dd _bss_start                            ;the end address of the data segment
    dd _kernel_end                           ;the end address of the bss segment
    dd start                                 ;the entry point for the kernel
multiboot_end:
