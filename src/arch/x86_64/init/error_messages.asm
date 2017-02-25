global no_multiboot_error
global no_long_mode_error
global no_SSE_error

section .data:
no_multiboot_error:
    db "Not loaded using a multiboot compliant bootloader. Consider using another bootloader. Aborting...",0
no_long_mode_error:
    db "Your CPU does not support 64 Bit mode. That means it's not possible to use this OS on your computer. Aborting...",0
no_SSE_error:
    db "Your CPU doesn't support SSE. This is required currently. Aborting...",0
