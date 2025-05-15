use memmap::MmapOptions;
use rayon::prelude::*;
use std::{
    fs,
    io::{self, BufRead},
    path::Path,
};

use crate::error::{WcError, WcResult};
use crate::parser::CountMode;

use proptest::arbitrary::Arbitrary;
use proptest::strategy::{Strategy, BoxedStrategy};
use proptest::prelude::any;

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

impl Arbitrary for WcCounter {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            0..1000usize,    // lines
            0..1000usize,    // words
            0..1000usize,    // bytes
            0..1000usize,    // chars
            0..1000usize,    // max_line_length
            proptest::option::of(any::<String>()) // filename
        )
            .prop_map(|(lines, words, bytes, chars, max_len, filename)| {
                // Enforce invariants
                let chars = chars.min(bytes);
                let max_len = max_len.min(bytes);
                
                WcCounter { 
                    lines, 
                    words: words.max(lines), // At least 1 word per line
                    bytes,
                    chars,
                    max_line_length: max_len,
                    filename 
                }
            })
            .boxed()
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


#[cfg(test)]
mod counter_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_wc_counter() {
        let counter = WcCounter::new();
        assert_eq!(counter.lines, 0);
        assert_eq!(counter.words, 0);
        assert_eq!(counter.bytes, 0);
        assert_eq!(counter.chars, 0);
        assert_eq!(counter.max_line_length, 0);
        assert!(counter.filename.is_none());
    }

    #[test]
    fn test_wc_counter_add_assign() {
        let mut counter1 = WcCounter {
            lines: 10,
            words: 20,
            bytes: 30,
            chars: 40,
            max_line_length: 50,
            filename: Some("file11".to_string()),
        };

        let counter2 = WcCounter {
            lines: 5,
            words: 15,
            bytes: 25,
            chars: 35,
            max_line_length: 60,
            filename: Some("file2".to_string()),
        };

        counter1 += &counter2;
        assert_eq!(counter1.lines, 15);
        assert_eq!(counter1.words, 35);
        assert_eq!(counter1.bytes, 55);
        assert_eq!(counter1.chars, 75);
        assert_eq!(counter1.max_line_length, 60);
        assert_eq!(counter1.filename, Some("file11".to_string()));
    }

    #[test]
    fn test_count_reader_empty() {
        let reader = Cursor::new(b"");
        let result = count_reader(reader, None, &[CountMode::Lines]).unwrap();
        assert_eq!(result.lines, 0);
    }

    #[test]
    fn test_count_reader_basic() {
        let text = "Hello world\nThis is a test\n";
        let reader = Cursor::new(text);
        let result = count_reader(reader, None, &[CountMode::Lines, CountMode::Words]).unwrap();
        
        assert_eq!(result.lines, 2);
        assert_eq!(result.words, 6);
    }

    #[test]
    fn test_count_bytes_unicode() {
        let text = "こんにちは世界\n"; // "Hello world" in Japanese
        let result = count_bytes(text.as_bytes(), None, &[CountMode::Chars]).unwrap();
        
        assert_eq!(result.chars, 8); // 7 characters + newline
    }

    #[test]
    fn test_count_file_not_found() {
        let result = count_file("/nonexistent/file", &[CountMode::Chars]);
        assert!(matches!(result, Err(WcError::FileNotFound(_))));
    }
}