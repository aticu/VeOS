global gdt
global gdt.code
global gdt.pointer

section .init
align 4096
gdt:
    dq 0 ;required
.code: equ $ - gdt
    dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53) ;executable, code, present, 64-bit
.gdt_end:
.pointer:
    dw .gdt_end - gdt - 1
    dq gdt
