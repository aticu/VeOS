global set_up_paging
global enable_paging

extern l4_table
extern l3_table
extern l2_table
extern stack_l2_table
extern stack_l1_table
extern stack_top
extern stack_bottom
extern STACK_SIZE
extern temporary_map

section .init
bits 32
set_up_paging: ;sets up very basic paging for the first GB of memory
.map_l4: ;map first entry to the currently only l4 table
    mov eax, l3_table
    or eax, 0b11 ;present + writable
    mov [l4_table], eax
    mov [l4_table + 256 * 8], eax ;map the high half addresses of the kernel
    mov [l4_table + 508 * 8], eax ;map the stack

    mov eax, temporary_map
    or eax, 0b11 ;present + writable
    mov [l4_table + 510 * 8], eax ;map the temporary map table

    mov eax, l4_table ;set up recursive mapping for the last page table entry
    or eax, 0b11 ;present + writable
    mov [l4_table + 511 * 8], eax

.map_l3: ;map first entry to the currently only l3 table
    mov eax, l2_table
    or eax, 0b11 ;present + writable
    mov [l3_table], eax

    mov eax, stack_l2_table
    or eax, 0b11 ;present + writable
    mov [l3_table + 511 * 8], eax ;map the stack

    mov ecx, 0

.map_l2: ;map all 512 entries as an identity
    mov eax, 0x200000 ;2 Mb
    mul ecx
    or eax, 0b10000011 ;present, writable and huge
    mov [l2_table + ecx * 8], eax

    inc ecx
    cmp ecx, 512
    jne .map_l2

.map_stack_l2: ;map last entry to the stack l1 table
    mov eax, stack_l1_table
    or eax, 0b11 ;present + writable
    mov [stack_l2_table + 511 * 8], eax
    ;NOTE: The stack must not be bigger than 2mb initially.

    ;calculate the number of the stack bottom within the l1 table
    mov ebx, stack_bottom
    mov eax, STACK_SIZE
    mov ecx, 0x1000
    div ecx
    mov ecx, 512
    sub ecx, eax

.map_stack_l1:
    mov eax, ebx
    or eax, 0b11 ;present + writable
    mov [stack_l1_table + ecx * 8], eax

    inc ecx
    add ebx, 0x1000 ;continue to the next page
    cmp ebx, stack_top
    jne .map_stack_l1

    ret

enable_paging:
    ;load base table
    mov eax, l4_table
    mov cr3, eax

    ;enable PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ;set long mode bit in EFER
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ;finally enable paging
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret
