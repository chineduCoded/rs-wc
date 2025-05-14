use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum CountMode {
    Lines,
    Words,
    Bytes,
    Chars,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Plain,
    Human,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "rs-wc",
    about = "A performant wc-like utility in Rust",
    version,
    long_about = "Counts lines, words, bytes, and characters in files or stdin.",
)]
pub struct Cli {
    /// Print the new line counts
    #[arg(short = 'l', long)]
    pub lines: bool,
    
    /// Print the word counts
    #[arg(short = 'w', long)]
    pub words: bool,
    
    /// Print the byte counts
    #[arg(short = 'c', long)]
    pub bytes: bool,
    
    /// Print the character counts
    #[arg(short = 'm', long)]
    pub chars: bool,
    
    /// Print maximum line length
    #[arg(short = 'L', long)]
    pub max_line_length: bool,
    
    /// Print all counts (lines, words, bytes)
    #[arg(short = 'a', long)]
    pub all: bool,
    
    /// Print output format (plain, human, json)
    #[arg(short = 'f', long, default_value = "plain")]
    pub format: OutputFormat,
    
    /// Input files (read from stdin if none specified)
    #[arg(value_name = "FILE", default_value = "-")]
    pub files: Vec<PathBuf>,
}

impl Default for Cli {
    fn default() -> Self {
        Self::parse()
    }
}

impl Cli {
    pub fn get_count_modes(&self) -> Vec<CountMode> {
        if self.all {
            return vec![CountMode::Lines, CountMode::Words, CountMode::Bytes];
        }

        let mut modes = Vec::new();
        
        if self.lines { modes.push(CountMode::Lines); }
        if self.words { modes.push(CountMode::Words); }
        if self.bytes { modes.push(CountMode::Bytes); }
        if self.chars { modes.push(CountMode::Chars); }

        if modes.is_empty() {
            vec![CountMode::Lines, CountMode::Words, CountMode::Bytes]
        } else {
            modes
        }
    }
}