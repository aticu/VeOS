//!Handles the elf symbols multiboot2 tag.

///Represents the elf symbols tag.
#[repr(C)]
struct ElfSymbols { //type = 9
    tag_type: u32,
    size: u32,
    num: u16,
    entsize: u16,
    shndx: u16,
    reserved: u16,
    section_headers: u32 //this is just a placeholder, the headers start here
}

