//! This modules aims to offer an abstraction for accessing files.

/// Abstracts the different kinds of errors that can occur with file operations.
#[derive(Debug)]
pub enum FileError {
    /// A seek before byte 0 was attempted.
    SeekBeforeStart,
    /// A seek after the last byte was attempted.
    SeekPastEnd
}

/// A result of a file operation.
pub type Result<T> = ::core::result::Result<T, FileError>;

/// The different ways to seek a file.
#[derive(Debug)]
pub enum SeekFrom {
    /// Seek from the start.
    Start(u64),
    /// Seek from the end.
    End(i64),
    /// Seek from the current seek position.
    Current(i64)
}

/// Everything that abstracts a file should implement this.
pub trait FileHandle {
    /// Sets the current seek position. Returns the offset from the beginning.
    fn seek(&mut self, position: SeekFrom) -> Result<u64>;

    /// Reads `length` bytes into `buffer`.
    fn read(&mut self, buffer: &mut [u8]) -> Result<()>;

    /// Reads `length` bytes into `buffer` at offset `position` from the
    /// beginning.
    fn read_at(&mut self, buffer: &mut [u8], position: u64) -> Result<()> {
        self.seek(SeekFrom::Start(position))
            .and_then(|_| self.read(buffer))
    }

    /// Returns the size of the file.
    fn len(&mut self) -> u64 {
        let current_seek = self.seek(SeekFrom::Current(0))
            .expect("Seeking at current position failed.");

        let size = self.seek(SeekFrom::End(0))
            .expect("Seek to end not possible.");

        self.seek(SeekFrom::Start(current_seek))
            .expect("Seeking to a previously valid location not possible.");

        size
    }
}
