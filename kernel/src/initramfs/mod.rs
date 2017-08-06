//! This modules is responsible for reading the initramfs.

use alloc::boxed::Box;
use arch::{get_initramfs_length, get_initramfs_start};
use core::ptr;
use file_handle::{FileError, FileHandle, Result, SeekFrom};
use memory::VirtualAddress;

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

/// Returns the file descriptor for the file with the given name or `None` if
/// it doesn't exist.
pub fn open(_: &str) -> Result<Box<FileHandle>> {
    Ok(Box::new(FileDescriptor {
                    start: get_initramfs_start(),
                    length: get_initramfs_length(),
                    current_offset: 0
                }))
}
