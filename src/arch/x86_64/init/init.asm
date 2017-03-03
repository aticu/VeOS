global long_mode_start

extern main

section .text
long_mode_start: ;first 64-bit code to be executed
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    call main

    ;in case the rust code ever returns, halt the CPU indefinitely
.endlessLoop:
    hlt
    jmp .endlessLoop
