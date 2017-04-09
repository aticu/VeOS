global stack_top
global gdt.code
global gdt.pointer

section .init
align 4096
gdt:
    dq 0 ;required
.code: equ $ - gdt
    dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53) ;executable, code, present, 64-bit
.pointer:
    dw $ - gdt - 1
    dq gdt
