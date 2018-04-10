#![feature(const_size_of)]

//! This crate is the initramfs creator for VeOS.

extern crate byteorder;

use std::env::args;
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{ErrorKind, SeekFrom};
use std::mem::{size_of, size_of_val};
use std::path::{Path, PathBuf};
use std::process::exit;

use byteorder::{BigEndian, WriteBytesExt};

/// Whether to force the creation of the initramfs.
const FORCE: bool = false;

/// Whether to overwrite the target if it exists.
const OVERWRITE: bool = true;

/// The magic number at the beginning of the output file.
const MAGIC: [u8; 8] = [
    'V' as u8, 'e' as u8, 'O' as u8, 'S' as u8, 'i' as u8, 'r' as u8, 'f' as u8, 's' as u8,
];

/// The offset at which the file metadata begins.
const FILE_METADATA_OFFSET: usize = size_of::<[u8; 8]>() + size_of::<u64>();

/// The size of a file metadata object.
const FILE_METADATA_SIZE: usize = size_of::<u64>() * 4;

/// The error message if there is a seek error.
const COULD_NOT_SEEK_TARGET: &str = "Could not seek target file";

/// The error message if there is a write error.
const COULD_NOT_WRITE_TO_TARGET: &str = "Could not write to target file";

/// The main entry point for the application.
fn main() {
    let config_path = if let Some(path) = args().nth(1) {
        if Path::new(&path).is_file() {
            path
        } else {
            print_usage("Config file not found.");
        }
    } else {
        print_usage("Not enough arguments supplied.");
    };

    let out_path = if let Some(path) = args().nth(2) {
        if !Path::new(&path).exists() || OVERWRITE || FORCE {
            path
        } else {
            print_usage("Target already exists.");
        }
    } else {
        print_usage("Not enough arguments supplied.");
    };

    let base_path = if let Some(path) = args().nth(3) {
        if Path::new(&path).is_dir() {
            path
        } else {
            print_usage("The supplied base path is not existant.");
        }
    } else {
        "/".to_string()
    };

    let content = get_content(&config_path).unwrap_or_exit("Error opening config file");

    let file_list = get_file_list(&base_path, &content);

    let mut file = File::create(out_path).unwrap_or_exit("Could not create target file");

    write_file_header(&mut file, &file_list).unwrap_or_exit(COULD_NOT_WRITE_TO_TARGET);

    for (file_num, &(ref original_path, ref actual_path)) in file_list.iter().enumerate() {
        write_file(&mut file, file_num, original_path, actual_path);
    }
}

/// Writes the file to the initramfs file.
///
/// The file name parameter specifies the name within the initramfs, while the file_path parameter specifies the path to the source file.
fn write_file(file: &mut File, file_num: usize, file_name: &str, file_path: &Path) {
    let file_metadata_start = FILE_METADATA_OFFSET + file_num * FILE_METADATA_SIZE;

    // Write file name.
    let name_position = file.seek(SeekFrom::End(0))
        .unwrap_or_exit(COULD_NOT_SEEK_TARGET);
    file.write(file_name.as_bytes())
        .unwrap_or_exit(COULD_NOT_WRITE_TO_TARGET);

    // Write file name metadata.
    file.seek(SeekFrom::Start(file_metadata_start as u64))
        .unwrap_or_exit(COULD_NOT_SEEK_TARGET);
    file.write_u64::<BigEndian>(name_position)
        .unwrap_or_exit(COULD_NOT_WRITE_TO_TARGET);
    file.write_u64::<BigEndian>(file_name.len() as u64)
        .unwrap_or_exit(COULD_NOT_WRITE_TO_TARGET);

    // Write file content.
    let content_position = file.seek(SeekFrom::End(0))
        .unwrap_or_exit(COULD_NOT_SEEK_TARGET);
    let mut source_file =
        File::open(file_path).unwrap_or_exit(&format!("Could not open {}", file_path.display()));

    let mut buffer = [0u8; 1024]; // Read a KiB at a time.
    loop {
        match source_file.read(&mut buffer) {
            Ok(0) => break,
            Ok(num) => {
                file.write_all(&buffer[0..num])
                    .unwrap_or_exit(COULD_NOT_WRITE_TO_TARGET);
            }
            Err(error) => match error.kind() {
                ErrorKind::Interrupted => (),
                _ => exit_with_error(&format!("Could not read {}", file_path.display()), error),
            },
        }
    }

    // Write file content metadata.
    file.seek(SeekFrom::Start(
        (file_metadata_start + size_of::<u64>() * 2) as u64,
    )).unwrap_or_exit(COULD_NOT_SEEK_TARGET);
    file.write_u64::<BigEndian>(content_position)
        .unwrap_or_exit(COULD_NOT_WRITE_TO_TARGET);
    file.write_u64::<BigEndian>(source_file
        .metadata()
        .unwrap_or_exit(&format!("Could not read length of {}", file_path.display()))
        .len() as u64)
        .unwrap_or_exit(COULD_NOT_WRITE_TO_TARGET);
}

/// Writes the header information to the file.
///
/// Returns the size of the header.
fn write_file_header(file: &mut File, file_list: &Vec<(&str, PathBuf)>) -> io::Result<u64> {
    // First write the magic number in the header.
    let mut bytes_written = 0;
    while bytes_written != size_of_val(&MAGIC) {
        bytes_written += file.write(&MAGIC[..])?;
    }

    // Next write the number of files (as a big endian u64).
    file.write_u64::<BigEndian>(file_list.len() as u64)?;

    // Now the files are listed in the following way:
    // First u64 (big endian) is the offset (from beginning) of the file name.
    // Second u64 (big endian) is the length of the file name.
    // Third u64 (big endian) is the offset (from beginning) of the file content.
    // Fourth u64 (big endian) is the length of the file content.

    // This function just reserves enough space for the file metadata.
    let header_len = FILE_METADATA_OFFSET + FILE_METADATA_SIZE * file_list.len();
    file.set_len(header_len as u64)?;

    Ok(header_len as u64)
}

/// Gets a list of all the valid files in the config file.
fn get_file_list<'a>(base_path: &String, content: &'a String) -> Vec<(&'a str, PathBuf)> {
    let base_path = Path::new(base_path);
    content
        .lines()
        .map(|line| (line, line.trim_matches('/')))
        .map(|(original_line, line)| (original_line, base_path.join(line)))
        .filter(|&(_, ref path)| {
            if !path.is_file() {
                eprintln!("File {} not found.", path.display());
                if !FORCE {
                    exit(1);
                } else {
                    return false;
                }
            }
            true
        })
        .collect()
}

/// Reads the file into a string.
fn get_content(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;

    Ok(content)
}

trait ExitOnError {
    type ResultType;

    fn unwrap_or_exit(self, message: &str) -> Self::ResultType;
}

impl<T, E: Display> ExitOnError for std::result::Result<T, E> {
    type ResultType = T;

    /// Either unwraps the result or exits with an error message.
    fn unwrap_or_exit(self, message: &str) -> T {
        match self {
            Ok(result) => result,
            Err(error) => exit_with_error(message, error),
        }
    }
}

/// Exits printing the error and the given message.
fn exit_with_error<E: Display>(message: &str, error: E) -> ! {
    print_usage(&format!("{}: {}", message, error));
}

/// Prints usage information and exits.
fn print_usage(error: &str) -> ! {
    eprintln!("{}", error);
    eprintln!("");
    eprintln!("Usage:");
    eprintln!("mkinitramfs config_path target_path [base_path]");
    eprintln!("    config_path is the path to the mkinitramfs configuration file.");
    eprintln!("    target_path is the path to the output file.");
    eprintln!("    base_path is the path that all the listed files start from. Default is \"/\".");
    exit(1)
}
