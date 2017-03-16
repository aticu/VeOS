global start

extern stack_top
extern check_multiboot
extern check_cpuid
extern check_long_mode
extern enable_SSE
extern set_up_paging
extern enable_paging
extern gdt.code
extern gdt.pointer
extern long_mode_start

section .text
bits 32
start:
    ;initialize stack pointer
    mov esp, stack_top
    mov esi, ebx ;save multiboot information address in esi for later use
    mov edi, eax ;save the multiboot magic number in edi for later use

    ;check if long mode is available
    call check_cpuid
    call check_long_mode
    call enable_SSE

    ;enable paging
    call set_up_paging
    call enable_paging

    ;load global descriptor table
    lgdt [gdt.pointer]

    jmp gdt.code:long_mode_start

.endlessLoop: ;shouldn't be reached
    hlt
    jmp .endlessLoop
