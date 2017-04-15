"to enable quickfix for cargo
compiler cargo
set makeprg=make

setlocal errorformat+=
            \%-G%\\s%#xargo%.%#,
            \%-G%\\s%#ld%.%#,
            \%-G%\\s%#make%.%#,
            \%-G%\\s%#grub%.%#,
            \%-G%\\s%#qemu%.%#,
            \%-G%\\s%#nasm%.%#

"autocmd QuickFixCmdPost [^l]* nested cwindow
"autocmd QuickFixCmdPost    l* nested lwindow
