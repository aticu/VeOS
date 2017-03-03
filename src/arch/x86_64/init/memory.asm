global stack_top
global pml4_table
global pdp_table
global pd_table
global gdt.code
global gdt.pointer

section .bss
align 4096
pml4_table:
    resb 4096
pdp_table:
    resb 4096
pd_table:
    resb 4096
stack_bottom:
    resb 4096 ;use a stack that has a page as it's size
stack_top:

section .rodata
gdt:
    dq 0 ;required
.code: equ $ - gdt
    dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53) ;executable, code, present, 64-bit
.pointer:
    dw $ - gdt - 1
    dq gdt
