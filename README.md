# VeOS
An experimental OS using Rust. Currently in a very early stage.

## Compiling
In order to compile VeOS you'll need the following:
- [Rust][5]
- xargo (can be installed with `cargo install xargo`, then you also need to run `rustup component add rust-src`)
- nasm
- ld
- grub (in order to make it bootable)

Then you can
- run `make` to create the folder structure of the OS at `target/`.
- run `make iso` to create a bootable image at `image.iso`.
- run `make run` to run the OS in qemu (if you have it installed).

## Acknowledgements
A lot of this work is based on work from the following people/organizations or at least highly influenced by it:
- Philipp Oppermann and his "[Writing an OS in Rust][1]" blog.
- The contributors of the [spin crate][2].
- Eric Kidd's [blog][3].
- The [OSDev wiki][4].
- The [Redox][6] project.
- Mike Rieker's excellent [APIC tutorial][7].

[1]: http://os.phil-opp.com/ "Writing an OS in Rust"
[2]: https://crates.io/crates/spin "The spin crate on crates.io"
[3]: http://www.randomhacks.net/bare-metal-rust/ "Bare Metal Rust: Building kernels in Rust"
[4]: http://wiki.osdev.org/Main_Page "OSDev wiki Main Page"
[5]: https://www.rust-lang.org/
[6]: https://www.redox-os.org
[7]: https://web.archive.org/web/20140308064246/http://www.osdever.net/tutorials/pdf/apic.pdf
