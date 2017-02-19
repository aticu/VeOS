section .multiboot_header
header_start:   ;start of multiboot header
    dd 0xe85250d6                                                       ;magic number for multiboot 2
    dd 0                                                                ;architechture protected mode i386
    dd header_end - header_start                                        ;length of header
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))     ;checksum

    ;optional tags here

    ;end tag
    dw 0
    dw 0
    dd 8
header_end:     ;end of multiboot header
