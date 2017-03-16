global check_cpuid
global check_long_mode

extern early_error
extern no_multiboot_error
extern no_long_mode_error

section .text
bits 32
check_cpuid:                ;check if the cpuid instruction is available
    ;move flags register to eax
    pushfd
    pop eax

    ;copy eax and flip bit 21
    mov ecx, eax
    xor eax, 1 << 21

    ;move eax to flags register
    push eax
    popfd

    ;move flags register back to eax
    pushfd
    pop eax

    ;check if the value changed
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov eax, no_long_mode_error
    jmp early_error

check_long_mode:            ;check if long mode is supported
    ;check for extended processor information
    mov eax, 1 << 31
    cpuid                   ;checks for the highest supported argument
    cmp eax, 1 << 31 | 1 << 0 ;if the argument is supported
    jb .no_long_mode        ;the argument is not supported -> no long mode

    mov eax, 1 << 31 | 1 << 0 ;argument for extended information
    cpuid
    test edx, 1 << 29       ;if bit 29 is set, long mode is supported
    jz .no_long_mode
    ret
.no_long_mode:
    mov eax, no_long_mode_error
    jmp early_error
