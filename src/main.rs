use clap::Parser;
use std::{
    io::{self, BufReader},
    path::PathBuf,
};

use rs_wc::{
    parser::Cli,
    error::WcResult,
    printer,
    counter::{self, count_files},
};

fn main() -> WcResult<()> {
    let cli = Cli::parse();
    
    let results = if cli.files.is_empty() || (cli.files.len() == 1 && cli.files[0] == PathBuf::from("-")) {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        vec![counter::count_reader(reader, None, &cli.get_count_modes())?]
    } else {
        count_files(&cli.files, &cli.get_count_modes())?
    };

    let output = printer::format_results(&results, &cli)?;
    print!("{}", output);

    Ok(())
}