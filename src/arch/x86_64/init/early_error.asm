global early_error

section .text
bits 32
early_error:
    mov ecx, 0
.loop:
    mov byte dl, [eax + ecx]
    mov byte dh, 0x4f
    cmp dl, 0
    je .end
    mov word [0xb8000 + ecx * 2], dx
    inc ecx
    jmp .loop
.end:
    hlt
