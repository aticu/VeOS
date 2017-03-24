ENTRY(start)
OUTPUT_FORMAT(elf64-x86-64)

PAGE_SIZE = 4K;
LOADER_START = 1M;
KERNEL_OFFSET = 0xffff800000000000;

SECTIONS {
    . = LOADER_START;

    .init :
    {
        KEEP(*(.multiboot_header))
        *(.init)
        . = ALIGN(PAGE_SIZE);
    }

    . += KERNEL_OFFSET;

    .text : AT(ADDR(.text) - KERNEL_OFFSET)
    {
        *(.text .text.*)
        . = ALIGN(PAGE_SIZE);
    }

    .rodata : AT(ADDR(.rodata) - KERNEL_OFFSET)
    {
        *(.rodata .rodata.*)
        . = ALIGN(PAGE_SIZE);
    }

    .data.rel.ro : AT(ADDR(.data.rel.ro) - KERNEL_OFFSET)
    {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(PAGE_SIZE);
    }

    .data : AT(ADDR(.data) - KERNEL_OFFSET)
    {
        *(.data .data.*)
        . = ALIGN(PAGE_SIZE);
    }

    _data_end = . - KERNEL_OFFSET;

    .bss : AT(ADDR(.bss) - KERNEL_OFFSET)
    {
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(PAGE_SIZE);
    }
    
    _bss_end = . - KERNEL_OFFSET;
}
