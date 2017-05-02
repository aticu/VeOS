ENTRY(start)
OUTPUT_FORMAT(elf64-x86-64)

PAGE_SIZE = CONSTANT(COMMONPAGESIZE);
STACK_SIZE = 16K;
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

    _text_start = . - KERNEL_OFFSET;

    .text : AT(ADDR(.text) - KERNEL_OFFSET)
    {
        *(.text .text.*)
        . = ALIGN(PAGE_SIZE);
    }

    _rodata_start = . - KERNEL_OFFSET;

    .rodata : AT(ADDR(.rodata) - KERNEL_OFFSET)
    {
        *(.rodata .rodata.*)
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(PAGE_SIZE);
    }

    _data_start = . - KERNEL_OFFSET;

    .data : AT(ADDR(.data) - KERNEL_OFFSET)
    {
        *(.data .data.*)
        TEXT_START = .;
        QUAD(_text_start);
        RODATA_START = .;
        QUAD(_rodata_start);
        DATA_START = .;
        QUAD(_data_start);
        BSS_START = .;
        QUAD(_data_end);
        BSS_END = .;
        QUAD(_bss_end);
        . = ALIGN(PAGE_SIZE);
    }

    _data_end = . - KERNEL_OFFSET;

    .bss : AT(ADDR(.bss) - KERNEL_OFFSET)
    {
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(PAGE_SIZE);
        l4_table = . - KERNEL_OFFSET;
        . += PAGE_SIZE;
        l3_table = . - KERNEL_OFFSET;
        . += PAGE_SIZE;
        l2_table = . - KERNEL_OFFSET;
        . += PAGE_SIZE;
        temporary_map = . - KERNEL_OFFSET;
        . += PAGE_SIZE;
        stack_bottom = . - KERNEL_OFFSET;
        . += STACK_SIZE;
        stack_top = . - KERNEL_OFFSET;
        . = ALIGN(PAGE_SIZE);
    }
    
    _bss_end = . - KERNEL_OFFSET;
}
