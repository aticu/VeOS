# VeOS
An experimental OS using Rust. Currently in a very early stage.

## Compiling
In order to compile VeOS you'll need the following:
- [Rust][5]
- xargo (can be installed with `cargo install xargo`, then you also need to run `rustup component add rust-src`)
- nasm
- ld
- grub (in order to make it bootable)

Run `make iso` to create a bootable cd image of the kernel at `target/*architecture*-unknonwn-none-gnu/build/os-*architecture*.iso`.
Or just run `make` to create the kernel binary at `target/*architecture*-unknonwn-none-gnu/build/kernel-*architecture*.bin`.

##Acknowledgements
A lot of this work is based on work from the following people/organizations or at least highly influenced by it:
- Philipp Oppermann and his "[Writing an OS in Rust][1]" blog.
- The contributors of the [spin crate][2].
- Eric Kidd for and his [blog][3].
- The [OSDev wiki][4].

[1]: http://os.phil-opp.com/ "Writing an OS in Rust"
[2]: https://crates.io/crates/spin "The spin crate on crates.io"
[3]: http://www.randomhacks.net/bare-metal-rust/ "Bare Metal Rust: Building kernels in Rust"
[4]: http://wiki.osdev.org/Main_Page "OSDev wiki Main Page"
[5]: https://www.rust-lang.org/
