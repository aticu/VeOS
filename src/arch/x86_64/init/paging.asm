global set_up_paging
global enable_paging

extern pml4_table
extern pdp_table
extern pd_table

section .text
bits 32
set_up_paging: ;sets up very basic paging for the first GB of memory
.map_pml4: ;map first entry to the only p3 table
    mov eax, pdp_table
    or eax, 0b11 ;present + writable
    mov [pml4_table], eax

.map_pdp: ;map first entry to the only p2 table
    mov eax, pd_table
    or eax, 0b11 ;present + writable
    mov [pdp_table], eax

    mov ecx, 0
.map_pd: ;map all 512 entries as an identity
    mov eax, 0x200000 ; 2 Mb
    mul ecx
    or eax, 0b10000011 ;present, writable and huge
    mov [pd_table + ecx * 8], eax

    inc ecx
    cmp ecx, 512
    jne .map_pd

    ret

enable_paging:
    ;load base table
    mov eax, pml4_table
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
