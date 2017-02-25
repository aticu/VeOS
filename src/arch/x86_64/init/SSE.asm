global enable_SSE

extern early_error
extern no_SSE_error

section .text
bits 32
enable_SSE:
    ;check if SSE is enabled
    mov eax, 0x1
    cpuid
    test edx, 1 << 25
    jz .no_SSE

    ;enable SSE
    mov eax, cr0
    and ax, 0xfffb ;clear coprocessor emulation
    or ax, 0x2     ;set coprocessor monitoring
    mov cr0, eax
    mov eax, cr4
    or ax, 3 << 9  ;set cr4.osfxsr, cr4.osxmmexcpt
    mov cr4, eax

    ret

.no_SSE:
    mov eax, no_SSE_error
    jmp early_error
