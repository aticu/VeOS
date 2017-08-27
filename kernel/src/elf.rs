//! Handles ELF files.

use alloc::boxed::Box;
use core::fmt;
use core::mem;
use core::mem::size_of;
use file_handle::FileHandle;
use initramfs;
use memory::{PAGE_SIZE, PhysicalAddress, VirtualAddress};
use memory::address_space;
use memory::address_space::{AddressSpace, Segment};
use multitasking::{create_process, ProcessID};

/// Represents an ELF file.
struct ElfFile {
    /// The handle to the file.
    file_handle: Box<FileHandle>,
    /// The header of the ELF file.
    header: Header
}

impl ElfFile {
    /// Reads an ELF file from the initramfs.
    fn from_initramfs(name: &str) -> Result<ElfFile, ElfError> {
        if let Ok(mut file_handle) = initramfs::open(name) {
            Header::from_file_handle(&mut *file_handle).and_then(|header| {
                let file_size = file_handle.len();

                // Check if the program header is fully contained in the file.
                if file_size <
                   (header.program_header_offset as u64)
                       .saturating_add((header.program_header_entry_num as u64)
                                           .saturating_mul(header.program_header_entry_size as
                                                           u64)) {
                    return Err(ElfError::InvalidFile);
                }

                // Check that all the program header segments are fully contained in the file.
                {
                    let program_header_iterator = ProgramHeaderIterator {
                        current_header_index: 0,
                        header_num: header.program_header_entry_num as usize,
                        header_size: header.program_header_entry_size as usize,
                        header_offset: header.program_header_offset as u64,
                        file_handle: &mut *file_handle
                    };

                    for program_header in program_header_iterator {
                        if !program_header.is_fully_contained(file_size) {
                            return Err(ElfError::InvalidFile);
                        }
                    }
                }

                Ok(ElfFile {
                       file_handle,
                       header
                   })
            })
        } else {
            Err(ElfError::FileNotExistant)
        }
    }

    /// Returns an iterator for the program header table.
    fn program_headers(&mut self) -> ProgramHeaderIterator {
        ProgramHeaderIterator {
            current_header_index: 0,
            header_num: self.header.program_header_entry_num as usize,
            header_size: self.header.program_header_entry_size as usize,
            header_offset: self.header.program_header_offset as u64,
            file_handle: &mut *self.file_handle
        }
    }
}

/// The possible types of errors that can occur while handling ELF files.
#[derive(Debug)]
pub enum ElfError {
    /// The file to load doesn't exist.
    FileNotExistant,
    /// The file is too short or doesn't contain a valid header.
    NotAnElfFile,
    /// The file is using an unknown ELF version.
    UnknownVersion,
    /// The file is of the wrong type.
    ///
    /// This could mean any of the following:
    /// - The file is for a different architecture.
    /// - The file has the wrong endianness.
    /// - The file is of the wrong ELF class for this architecture.
    /// - The file is not executable.
    WrongType,
    /// The file is not a valid ELF file.
    InvalidFile,
    /// The segments within the ELF file overlapped.
    OverlappingSegments
}

/// Differentiates the endianness (byte order).
#[repr(u8)]
#[derive(Debug, PartialEq)]
enum Endianness {
    /// Least significant byte first.
    Little = 1,
    /// Most significant byte first.
    Big = 2
}

impl Endianness {
    /// Returns true if the endianness is the same as the one of the kernel.
    fn is_native(&self) -> bool {
        if cfg!(target_endian = "little") {
            *self == Endianness::Little
        } else if cfg!(target_endian = "big") {
            *self == Endianness::Big
        } else {
            false
        }
    }
}

/// Differentiates between 32 and 64 bit executables.
#[repr(u8)]
#[derive(Debug, PartialEq)]
enum ELFClass {
    /// A 32 bit exectuable.
    Bit32 = 1,
    /// A 64 bit executable.
    Bit64 = 2
}

impl ELFClass {
    /// Checks if the bus width for the binary is the same as in the kernel.
    fn is_native(&self) -> bool {
        if cfg!(target_pointer_width = "64") {
            *self == ELFClass::Bit64
        } else if cfg!(target_pointer_width = "32") {
            *self == ELFClass::Bit32
        } else {
            false
        }
    }
}

/// The different types of ELF files.
#[repr(u16)]
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum ElfType {
    /// No file type is specified.
    NoFileType = 0,
    /// The ELF file is suitable for further linking.
    Relocatable = 1,
    /// The ELF file may be executed.
    Executable = 2,
    /// The ELF file is a shared object.
    Shared = 3,
    /// File contents are unspecified (reserved).
    Core = 4
}

/// The instruction set of the executable file.
#[repr(u16)]
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum InstructionSet {
    /// No instruction set was given.
    NoSpecific = 0,
    /// The ELF file uses the x86 instruction set.
    #[allow(non_camel_case_types)]
    x86 = 3,
    /// The ELF file uses the ARM instruction set.
    ARM = 0x28,
    /// The ELF file uses the x86_64 instruction set.
    #[allow(non_camel_case_types)]
    x86_64 = 0x3e
}

impl InstructionSet {
    /// Returns true if the instruction set corresponds to the instruction set
    /// of the machine.
    fn is_native(&self) -> bool {
        if cfg!(target_arch = "x86_64") {
            *self == InstructionSet::x86_64
        } else {
            false
        }
    }
}

/// Represents the header at the beginning of an ELF file.
#[repr(C, packed)]
struct Header {
    /// The magic number: [0x7f, 'E', 'L', 'F'].
    magic: [u8; 4],
    /// The class of ELF file.
    elf_class: ELFClass,
    /// The endianness of the ELF file.
    endianness: Endianness,
    /// The ELF version.
    version: u8,
    /// The ABI of the file.
    abi: u8,
    /// The ABI version of the file.
    abi_version: u8,
    /// Unused padding bytes.
    padding: [u8; 7],
    /// The type of ELF file.
    elf_type: ElfType,
    /// The instruction set used by the ELF file.
    instruction_set: InstructionSet,
    /// The version of the ELF file.
    elf_version: u32,
    /// The entry address for the program.
    program_entry: VirtualAddress,
    /// The offset from file start to the program header.
    program_header_offset: usize,
    /// The offset from file start to the section header.
    section_header_offset: usize,
    /// Architecture specific flags.
    flags: u32,
    /// The size of the ELF header.
    header_size: u16,
    /// The size of a program header entry.
    program_header_entry_size: u16,
    /// The amount of program header entries.
    program_header_entry_num: u16,
    /// The size of a section header entry.
    section_header_entry_size: u16,
    /// The amount of section header entries.
    section_header_entry_num: u16,
    /// The index of the section header to the name strings.
    name_string_table_index: u16
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.is_valid() {
            write!(f, "Invalid ELF header.")
        } else {
            write!(f,
                   "Magic: ['\\x{:x}', '{}', '{}', '{}'],
{}: {:?}, {}: {:?}, {}: {}, {}: {:?}, {}: {}, {}: {:x}",
                   self.magic[0],
                   self.magic[1] as char,
                   self.magic[2] as char,
                   self.magic[3] as char,
                   "ELFClass",
                   self.elf_class,
                   "Endianness",
                   self.endianness,
                   "ABI",
                   self.abi,
                   "Type",
                   self.elf_type,
                   "Version",
                   self.elf_version,
                   "Entry address",
                   self.program_entry)
        }
    }
}

impl Header {
    /// Creates a ELF header from the file handle.
    fn from_file_handle(file_handle: &mut FileHandle) -> Result<Header, ElfError> {
        let file_size = file_handle.len();

        if file_size < size_of::<Header>() as u64 {
            return Err(ElfError::NotAnElfFile);
        }

        let header: Header = unsafe {
            let mut header_buffer: [u8; size_of::<Header>()] = mem::uninitialized();

            file_handle.read(&mut header_buffer).unwrap();

            mem::transmute(header_buffer)
        };

        if !header.is_valid() {
            return Err(ElfError::NotAnElfFile);
        }

        if header.version != 1 {
            return Err(ElfError::UnknownVersion);
        }

        if !header.is_executable() {
            return Err(ElfError::WrongType);
        }

        Ok(header)
    }

    /// Returns true if the header is valid and can be understood by the kernel.
    fn is_valid(&self) -> bool {
        self.magic == [0x7f, 'E' as u8, 'L' as u8, 'F' as u8]
    }

    /// Returns true if the file can be executed.
    fn is_executable(&self) -> bool {
        self.endianness.is_native() && self.instruction_set.is_native() && self.abi == 0 &&
        self.abi_version == 0 &&
        self.elf_type == ElfType::Executable && self.program_header_offset != 0 &&
        self.elf_class.is_native()
    }
}

/// Represents the different segment types in the program header.
#[repr(u32)]
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum SegmentType {
    /// An unused entry.
    Null = 0,
    /// A loadable segment.
    Load = 1,
    /// Dynamic linking tables.
    Dynamic = 2,
    /// The path to a program interpreter.
    Interpreter = 3,
    /// Note sections.
    Note = 4
}

bitflags! {
    /// The different flags a segment can have.
    flags SegmentFlags: u32 {
        /// The segment may be executed.
        const EXECUTABLE = 0x1,
        /// The segment is writable.
        const WRITABLE = 0x2,
        /// The segment is readable.
        const READABLE = 0x4
    }
}

/// Represents the program header of an ELF file.
#[repr(C, packed)]
#[derive(Debug)]
struct ProgramHeader {
    /// The type of the segment.
    segment_type: SegmentType,
    /// The flags of the segment.
    flags: SegmentFlags,
    /// The offset from the beginning of the file.
    offset: usize,
    /// The virtual address at which the segment should be mapped.
    virtual_address: VirtualAddress,
    /// The physical address at which the segment should be mapped.
    ///
    /// # Note
    /// Reserved for systems with physical addressing.
    physical_address: PhysicalAddress,
    /// The size of the segment within the file.
    size_in_file: usize,
    /// The size of the segment within memory.
    size_in_memory: usize,
    /// The alignment of the segment.
    align: usize
}

impl ProgramHeader {
    fn is_fully_contained(&self, file_size: u64) -> bool {
        file_size >= (self.offset as u64).saturating_add(self.size_in_file as u64) || self.size_in_file == 0
    }
}

/// Provides an iterator for the program headers.
struct ProgramHeaderIterator<'a> {
    /// The index of the current program header.
    current_header_index: usize,
    /// The amount of program headers in the table.
    header_num: usize,
    /// The size of a single program header.
    header_size: usize,
    /// The offset of the program header table in the file.
    header_offset: u64,
    /// The handle to the ELF file.
    file_handle: &'a mut FileHandle
}

impl<'a> Iterator for ProgramHeaderIterator<'a> {
    type Item = ProgramHeader;

    fn next(&mut self) -> Option<ProgramHeader> {
        if self.current_header_index >= self.header_num {
            None
        } else {
            let program_header: ProgramHeader = unsafe {
                let mut program_header_buffer: [u8; size_of::<ProgramHeader>()] =
                    mem::uninitialized();

                self.file_handle
                    .read_at(&mut program_header_buffer,
                             self.header_offset +
                             self.header_size as u64 * self.current_header_index as u64)
                    .unwrap();

                mem::transmute(program_header_buffer)
            };

            self.current_header_index += 1;

            Some(program_header)
        }
    }
}

/// Creates a new process from the given file on the initramfs.
pub fn process_from_initramfs_file(name: &str) -> Result<ProcessID, ElfError> {
    ElfFile::from_initramfs(name).and_then(|file| process_from_elf_file(file))
}

/// Creates a new process from the given ELF file handle.
fn process_from_elf_file(mut file: ElfFile) -> Result<ProcessID, ElfError> {
    let mut address_space = AddressSpace::new();

    {
        let mut iterator = file.program_headers();

        // For each segment.
        while let Some(program_header) = iterator.next() {
            if program_header.segment_type != SegmentType::Load {
                continue;
            }

            // Convert the flags to page flags.
            let mut flags = ::memory::USER_ACCESSIBLE;

            if program_header.flags.contains(READABLE) {
                flags |= ::memory::READABLE;
            }

            if program_header.flags.contains(WRITABLE) {
                flags |= ::memory::WRITABLE;
            }

            if program_header.flags.contains(EXECUTABLE) {
                flags |= ::memory::EXECUTABLE;
            }

            let segment = Segment::new(program_header.virtual_address,
                                       program_header.size_in_memory,
                                       flags,
                                       address_space::SegmentType::FromFile);

            if !address_space.add_segment(segment) {
                return Err(ElfError::OverlappingSegments);
            }

            // Map all the segments (page by page).
            let pages_in_file = if program_header.size_in_file != 0 {
                (program_header.size_in_file - 1) / PAGE_SIZE + 1
            } else {
                0
            };
            for i in 0..pages_in_file {
                let mut segment_data_buffer: [u8; ::memory::PAGE_SIZE] =
                    unsafe { mem::uninitialized() };

                let segment_data = if program_header.size_in_file < (i + 1) * PAGE_SIZE {
                    &mut segment_data_buffer[0..program_header.size_in_file % PAGE_SIZE]
                } else {
                    &mut segment_data_buffer[..]
                };

                let read_result = iterator
                    .file_handle
                    .read_at(segment_data, (program_header.offset + i * PAGE_SIZE) as u64);

                if read_result.is_err() {
                    return Err(ElfError::InvalidFile);
                }

                address_space.write_to(segment_data,
                                       program_header.virtual_address + i * PAGE_SIZE);
            }

            let pages_in_memory = (program_header.size_in_memory - 1) / PAGE_SIZE + 1;
            for i in pages_in_file..pages_in_memory {
                address_space.map_page(program_header.virtual_address + i * PAGE_SIZE);
            }
        }
    }

    Ok(create_process(address_space, file.header.program_entry))
}
