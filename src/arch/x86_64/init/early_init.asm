global start

extern stack_top
extern check_multiboot
extern check_cpuid
extern check_long_mode
extern set_up_paging
extern enable_paging
extern gdt.code
extern gdt.pointer
extern long_mode_start

section .init
bits 32
start:
    ;disable interrupts
    cli

    ;initialize stack pointer
    mov esp, stack_top
    mov esi, ebx ;save multiboot information address in esi for later use
    mov edi, eax ;save the multiboot magic number in edi for later use

    mov dword [0xb8000], 0x2f4b2f4f
    hlt

    ;check if long mode is available
    call check_cpuid
    call check_long_mode

    ;enable paging
    call set_up_paging
    call enable_paging

    ;load global descriptor table
    lgdt [gdt.pointer]

    jmp gdt.code:long_mode_start

.endlessLoop: ;shouldn't be reached
    hlt
    jmp .endlessLoop
