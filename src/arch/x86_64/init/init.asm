global long_mode_start

extern main

section .init
long_mode_start: ;first 64-bit code to be executed
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    add rsi, 0xffff800000000000

    mov rax, main
    jmp rax

    ;in case the rust code ever returns, halt the CPU indefinitely
.endlessLoop:
    hlt
    jmp .endlessLoop
