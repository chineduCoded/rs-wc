use memmap::MmapOptions;
use rayon::prelude::*;
use std::{
    fs,
    io::{self, BufRead},
    path::Path,
};

use crate::error::{WcError, WcResult};
use crate::parser::CountMode;

#[derive(Debug, Default, Clone)]
pub struct WcCounter {
    pub lines: usize,
    pub words: usize,
    pub bytes: usize,
    pub chars: usize,
    pub max_line_length: usize,
    pub filename: Option<String>,
}

impl WcCounter {
    pub fn new() -> Self {
        Self::default()
    }

    // Helper method to add counts from another counter
    pub fn add_counts(&mut self, other: &WcCounter) {
        self.lines += other.lines;
        self.words += other.words;
        self.bytes += other.bytes;
        self.chars += other.chars;
        self.max_line_length = self.max_line_length.max(other.max_line_length);
    }
}

impl std::ops::AddAssign<&WcCounter> for WcCounter {
    fn add_assign(&mut self, other: &WcCounter) {
        self.add_counts(other);
    }
}

// Common counting logic extracted to a separate function
fn process_chunk(chunk: &[u8], initial_in_word: bool, initial_line_length: usize) -> WcCounter {
    let mut partial = WcCounter::new();
    let mut in_word = initial_in_word;
    let mut current_line_length = initial_line_length;

    for &byte in chunk {
        current_line_length += 1;
        
        if byte == b'\n' {
            partial.lines += 1;
            partial.max_line_length = partial.max_line_length.max(current_line_length);
            current_line_length = 0;
        }
        
        if byte.is_ascii_whitespace() {
            if in_word {
                partial.words += 1;
            }
            in_word = false;
        } else {
            in_word = true;
            partial.chars += 1;
        }
    }

    partial
}

pub fn count_file<P: AsRef<Path>>(
    path: P,
    modes: &[CountMode],
) -> WcResult<WcCounter> {
    let path = path.as_ref();
    let filename = path.to_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| path.display().to_string());

    if path == Path::new("-") {
        return count_reader(io::stdin().lock(), Some(filename), modes);
    }

    let file = fs::File::open(path)
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => WcError::file_not_found(&filename),
            io::ErrorKind::PermissionDenied => WcError::permission_denied(&filename),
            _ => WcError::Io(e),
        })?;
    
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    count_bytes(&mmap, Some(filename), modes)
}

pub fn count_reader<R: BufRead>(
    mut reader: R,
    filename: Option<String>,
    modes: &[CountMode],
) -> WcResult<WcCounter> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    count_bytes(&buffer, filename, modes)
}

pub fn count_bytes(
    bytes: &[u8],
    filename: Option<String>,
    modes: &[CountMode],
) -> WcResult<WcCounter> {
    let mut counter = WcCounter {
        filename,
        ..Default::default()
    };

    if modes.contains(&CountMode::Bytes) {
        counter.bytes = bytes.len();
    }

    if modes.iter().any(|m| matches!(m, CountMode::Lines | CountMode::Words | CountMode::Chars)) {
        // Process chunks in parallel for large files
        const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB
        
        let chunks = bytes.par_chunks(CHUNK_SIZE);
        let partial_counts: Vec<_> = chunks
            .map(|chunk| process_chunk(chunk, false, 0))
            .collect();

        for partial in &partial_counts {
            counter += partial;
        }

        // Handle potential partial word at the end
        if bytes.last().map_or(false, |&b| !b.is_ascii_whitespace()) {
            counter.words += 1;
        }

        if modes.contains(&CountMode::Chars) {
            counter.chars = match std::str::from_utf8(bytes) {
                Ok(s) => s.chars().count(),
                Err(_) => bytes.len(),
            }
        }
    }

    Ok(counter)
}

pub fn count_files<P: AsRef<Path> + Sync>(
    paths: &[P],
    modes: &[CountMode]
) -> WcResult<Vec<WcCounter>> {
    paths.par_iter()
        .map(|path| count_file(path, modes))
        .collect()
}