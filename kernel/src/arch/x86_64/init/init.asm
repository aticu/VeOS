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

    mov rsp, 0xfffffe8000000000 ;make the stack pointer point to the virtual stack top

    mov rax, main
    jmp rax

    ;in case the rust code ever returns, halt the CPU indefinitely
.endlessLoop:
    cli
    hlt
    jmp .endlessLoop
