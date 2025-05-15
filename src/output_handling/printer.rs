use crate::{
    parser::{Cli, CountMode, OutputFormat},
    counter::WcCounter,
    error::WcResult,
};
use serde_json::{json, to_string_pretty};

// Common trait for formatting counts
trait CountFormatter {
    fn format_count(&self, mode: &CountMode, counter: &WcCounter) -> String;
    fn format_max_line_length(&self, counter: &WcCounter) -> String;
    fn format_filename(&self, filename: &Option<String>) -> String;
    fn format_total_label(&self) -> String;
}

struct PlainFormatter;
struct HumanFormatter;

impl CountFormatter for PlainFormatter {
    fn format_count(&self, mode: &CountMode, counter: &WcCounter) -> String {
        match mode {
            CountMode::Lines => counter.lines.to_string(),
            CountMode::Words => counter.words.to_string(),
            CountMode::Bytes => counter.bytes.to_string(),
            CountMode::Chars => counter.chars.to_string(),
        }
    }

    fn format_max_line_length(&self, counter: &WcCounter) -> String {
        counter.max_line_length.to_string()
    }

    fn format_filename(&self, filename: &Option<String>) -> String {
        filename.as_deref().unwrap_or("").to_string()
    }

    fn format_total_label(&self) -> String {
        "total".to_string()
    }
}

impl CountFormatter for HumanFormatter {
    fn format_count(&self, mode: &CountMode, counter: &WcCounter) -> String {
        match mode {
            CountMode::Lines => format!("lines: {}", counter.lines),
            CountMode::Words => format!("words: {}", counter.words),
            CountMode::Bytes => format!("bytes: {}", counter.bytes),
            CountMode::Chars => format!("chars: {}", counter.chars),
        }
    }

    fn format_max_line_length(&self, counter: &WcCounter) -> String {
        format!("{} max line length", counter.max_line_length)
    }

    fn format_filename(&self, filename: &Option<String>) -> String {
        filename.as_ref()
            .map(|f| format!("in {}", f))
            .unwrap_or_default()
    }

    fn format_total_label(&self) -> String {
        "total".to_string()
    }
}

fn build_output<F: CountFormatter>(
    results: &[WcCounter],
    cli: &Cli,
    formatter: F,
) -> String {
    let modes = cli.get_count_modes();
    let mut output = String::new();

    for result in results {
        let mut parts: Vec<String> = modes.iter()
            .map(|mode| formatter.format_count(mode, result))
            .collect();

        if cli.max_line_length {
            parts.push(formatter.format_max_line_length(result));
        }

        if let Some(filename) = &result.filename {
            parts.push(formatter.format_filename(&Some(filename.clone())));
        }

        output.push_str(&parts.join(" "));
        output.push('\n');
    }

    if results.len() > 1 {
        let mut total = WcCounter::new();
        for result in results {
            total += result;
        }

        let mut parts: Vec<String> = modes.iter()
            .map(|mode| formatter.format_count(mode, &total))
            .collect();

        if cli.max_line_length {
            parts.push(formatter.format_max_line_length(&total));
        }

        parts.push(formatter.format_total_label());
        output.push_str(&parts.join(" "));
        output.push('\n');
    }

    output
}

fn format_json(results: &[WcCounter], cli: &Cli) -> WcResult<String> {
    let modes = cli.get_count_modes();
    let mut json_results = Vec::with_capacity(results.len() + 1);

    for result in results {
        let mut json_obj = serde_json::Map::new();

        for mode in &modes {
            match mode {
                CountMode::Lines => json_obj.insert("lines".into(), json!(result.lines)),
                CountMode::Words => json_obj.insert("words".into(), json!(result.words)),
                CountMode::Bytes => json_obj.insert("bytes".into(), json!(result.bytes)),
                CountMode::Chars => json_obj.insert("chars".into(), json!(result.chars)),
            };
        }

        if cli.max_line_length {
            json_obj.insert("max_line_length".into(), json!(result.max_line_length));
        }

        if let Some(filename) = &result.filename {
            json_obj.insert("filename".into(), json!(filename));
        }

        json_results.push(json!(json_obj));
    }

    if results.len() > 1 {
        let mut total = WcCounter::new();
        for result in results {
            total += result;
        }

        let mut json_obj = serde_json::Map::new();
        for mode in &modes {
            match mode {
                CountMode::Lines => json_obj.insert("lines".into(), json!(total.lines)),
                CountMode::Words => json_obj.insert("words".into(), json!(total.words)),
                CountMode::Bytes => json_obj.insert("bytes".into(), json!(total.bytes)),
                CountMode::Chars => json_obj.insert("chars".into(), json!(total.chars)),
            };
        }

        if cli.max_line_length {
            json_obj.insert("max_line_length".into(), json!(total.max_line_length));
        }

        json_obj.insert("type".into(), json!("total"));
        json_results.push(json!(json_obj));
    }

    to_string_pretty(&json_results).map_err(Into::into)
}

pub fn format_results(results: &[WcCounter], cli: &Cli) -> WcResult<String> {
    if cli.max_line_length && !cli.lines && !cli.words && !cli.bytes && !cli.chars {
        return Ok(results.iter().map(|r| {
            format!("{} {}\n", r.max_line_length, r.filename.as_deref().unwrap_or(""))
        }).collect::<Vec<_>>().join("\n"));
    }

    match cli.format {
        OutputFormat::Plain => Ok(build_output(results, cli, PlainFormatter)),
        OutputFormat::Human => Ok(build_output(results, cli, HumanFormatter)),
        OutputFormat::Json => format_json(results, cli),
    }
}

#[cfg(test)]
mod printer_tests {
    use super::*;
    use crate::counter::WcCounter;
    use crate::parser::{Cli, OutputFormat};

    fn create_test_counter() -> WcCounter {
        WcCounter {
            lines: 10,
            words: 20,
            bytes: 30,
            chars: 40,
            max_line_length: 50,
            filename: Some("test.txt".to_string()),
        }
    }

    #[test]
    fn test_format_plain_single() {
        let counter = create_test_counter();
        let cli = Cli {
            lines: true,
            words: true,
            bytes: true,
            ..Cli::default()
        };
        
        let output = build_output(&[counter], &cli, PlainFormatter);
        assert_eq!(output.trim(), "10 20 30 test.txt");
    }

    #[test]
    fn test_format_plain_multiple() {
        let counter1 = create_test_counter();
        let counter2 = WcCounter {
            lines: 5,
            words: 10,
            bytes: 15,
            chars: 20,
            max_line_length: 25,
            filename: Some("test2.txt".to_string()),
        };
        
        let cli = Cli {
            lines: true,
            words: true,
            ..Cli::default()
        };
        
        let output = build_output(&[counter1, counter2], &cli, PlainFormatter);
        let lines: Vec<&str> = output.trim().lines().collect();
        
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("test.txt"));
        assert!(lines[1].contains("test2.txt"));
        assert!(lines[2].contains("total"));
    }

    #[test]
    fn test_format_human() {
        let counter = create_test_counter();
        let cli = Cli {
            lines: true,
            words: true,
            format: OutputFormat::Human,
            ..Cli::default()
        };
        
        let output = build_output(&[counter], &cli, HumanFormatter);
        assert!(output.contains("lines: 10"));
        assert!(output.contains("words: 20"));
        assert!(output.contains("in test.txt"));
    }

    #[test]
    fn test_format_json() {
        let counter = create_test_counter();
        let cli = Cli {
            lines: true,
            format: OutputFormat::Json,
            ..Cli::default()
        };
        
        let output = format_json(&[counter], &cli).unwrap();
        assert!(output.contains("\"lines\": 10"));
        assert!(output.contains("\"filename\": \"test.txt\""));
    }
}