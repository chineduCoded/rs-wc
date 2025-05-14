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
    match cli.format {
        OutputFormat::Plain => Ok(build_output(results, cli, PlainFormatter)),
        OutputFormat::Human => Ok(build_output(results, cli, HumanFormatter)),
        OutputFormat::Json => format_json(results, cli),
    }
}