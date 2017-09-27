//! This modules is responsible for reading the initramfs.

use alloc::boxed::Box;
use arch::{get_initramfs_length, get_initramfs_start};
use core::{ptr, slice, str};
use core::mem::size_of;
use file_handle::{FileError, FileHandle, Result, SeekFrom};
use memory::VirtualAddress;

/// The magic number that identifies a VeOS initramfs.
const MAGIC: [u8; 8] = ['V' as u8,
                        'e' as u8,
                        'O' as u8,
                        'S' as u8,
                        'i' as u8,
                        'r' as u8,
                        'f' as u8,
                        's' as u8];

/// The size of a single metadata object within the initramfs.
const FILE_METADATA_SIZE: usize = size_of::<u64>() * 4;

/// Represents a file in the initramfs.
pub struct FileDescriptor {
    /// The start address of the file.
    start: VirtualAddress,
    /// The length of the file.
    length: usize,
    /// The current offset within the file.
    current_offset: u64
}

impl FileHandle for FileDescriptor {
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        match position {
            SeekFrom::Start(offset) => {
                if offset > self.length as u64 {
                    Err(FileError::SeekPastEnd)
                } else {
                    self.current_offset = offset;
                    Ok(self.current_offset)
                }
            },
            SeekFrom::Current(offset) => {
                if offset == <i64>::min_value() {
                    // The minimum value cannot be inverted, making it a special case.
                    let offset = (-(offset + 1)) as u64 + 1;

                    if offset > self.current_offset {
                        Err(FileError::SeekBeforeStart)
                    } else {
                        self.current_offset -= offset;
                        Ok(self.current_offset)
                    }
                } else if offset > 0 {
                    if self.current_offset.saturating_add(offset as u64) > self.length as u64 {
                        Err(FileError::SeekPastEnd)
                    } else {
                        self.current_offset = self.current_offset.saturating_add(offset as u64);
                        Ok(self.current_offset)
                    }
                } else if (-offset) as u64 > self.current_offset {
                    Err(FileError::SeekBeforeStart)
                } else {
                    self.current_offset = self.current_offset.saturating_sub((-offset) as u64);
                    Ok(self.current_offset)
                }
            },
            SeekFrom::End(offset) => {
                if offset == <i64>::min_value() {
                    // The minimum value cannot be inverted, making it a special case.
                    let offset = (-(offset + 1)) as u64 + 1;

                    if offset > self.length as u64 {
                        Err(FileError::SeekBeforeStart)
                    } else {
                        self.current_offset = self.length as u64 - offset;
                        Ok(self.current_offset)
                    }
                } else if offset > 0 {
                    Err(FileError::SeekPastEnd)
                } else if ((-offset) as usize) > self.length {
                    Err(FileError::SeekBeforeStart)
                } else {
                    self.current_offset = (self.length as u64).saturating_sub((-offset) as u64);
                    Ok(self.current_offset)
                }
            },
        }
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<()> {
        if self.current_offset.saturating_add(buffer.len() as u64) > self.length as u64 {
            Err(FileError::SeekPastEnd)
        } else {
            let source = unsafe { &*((self.start + self.current_offset as usize) as *const u8) };
            unsafe {
                ptr::copy_nonoverlapping(source, buffer.as_mut_ptr(), buffer.len());
            }
            Ok(())
        }
    }
}

/// Represents the metadata of a file.
struct FileMetadata {
    /// The name of the file.
    name: &'static str,
    /// The start address of the file data.
    start: VirtualAddress,
    /// The length of the file.
    length: usize
}

/// An iterator through the file metadata.
struct FileIterator {
    /// The address of the file metadata that is returned next.
    current_file_metadata_address: VirtualAddress,
    /// The address of the highest file number that can be returned.
    max_address: VirtualAddress
}

impl Iterator for FileIterator {
    type Item = FileMetadata;

    fn next(&mut self) -> Option<FileMetadata> {
        let start = get_initramfs_start();
        let length = get_initramfs_length();

        loop {
            if self.current_file_metadata_address < self.max_address {
                let name_offset = unsafe { read_u64_big_endian(self.current_file_metadata_address) };

                self.current_file_metadata_address += size_of::<u64>();

                let name_length = unsafe { read_u64_big_endian(self.current_file_metadata_address) };

                self.current_file_metadata_address += size_of::<u64>();

                let content_offset = unsafe { read_u64_big_endian(self.current_file_metadata_address) };

                self.current_file_metadata_address += size_of::<u64>();

                let content_length = unsafe { read_u64_big_endian(self.current_file_metadata_address) };

                self.current_file_metadata_address += size_of::<u64>();

                if name_offset + name_length <= length as u64 && content_offset + content_length <= length as u64 {
                    let name_slice = unsafe { slice::from_raw_parts((start + name_offset as usize) as *const u8, name_length as usize) };
                    let name = str::from_utf8(name_slice);

                    if name.is_err() {
                        continue;
                    }

                    break Some(FileMetadata {
                        name: name.unwrap(),
                        start: start + content_offset as usize,
                        length: content_length as usize
                    });
                }
            } else {
                break None;
            }
        }
    }
}

/// Returns an iterator through the file metadata.
fn get_file_iterator() -> Result<FileIterator> {
    if !initramfs_valid() {
        Err(FileError::InvalidFilesystem)
    } else {
        let start = get_initramfs_start();

        let first_metadata = start + size_of::<[u8; 8]>() + size_of::<u64>();
        let amount_of_files = unsafe { read_u64_big_endian(start + size_of::<[u8; 8]>()) } as usize;

        Ok(FileIterator {
            current_file_metadata_address: first_metadata,
            max_address: first_metadata + FILE_METADATA_SIZE * amount_of_files
        })
    }
}

/// Reads the u64 at the given address.
///
/// # Safety
/// - Make sure that the address is contained within the initramfs.
unsafe fn read_u64_big_endian(address: VirtualAddress) -> u64 {
    let bytes: [u8; size_of::<u64>()] = *(address as *const [u8; size_of::<u64>()]);
    let mut result: u64 = 0;

    for i in 0..size_of::<u64>() {
        result |= (bytes[i] as u64) << ((size_of::<u64>() - i - 1) * 8);
    }

    result
}

/// Checks whether the initramfs is valid.
fn initramfs_valid() -> bool {
    let start = get_initramfs_start();
    let length = get_initramfs_length();

    if length < size_of::<[u8; 8]>() + size_of::<u64>() {
        false
    } else {
        let magic_in_file: [u8; 8] = unsafe { *(start as *const [u8; 8]) };

        let amount_of_files = unsafe { read_u64_big_endian(start + size_of::<[u8; 8]>()) };

        magic_in_file == MAGIC && length >= size_of::<[u8; 8]>() + size_of::<u64>() + FILE_METADATA_SIZE * amount_of_files as usize
    }
}

/// Returns the file descriptor for the file with the given name.
pub fn open(name: &str) -> Result<Box<FileHandle>> {
    for file in get_file_iterator()? {
        if file.name == name {
            return Ok(Box::new(FileDescriptor { start: file.start, length: file.length, current_offset: 0 }));
        }
    }

    Err(FileError::FileNotFound)
}
