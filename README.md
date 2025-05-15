# Rust wc-like utilty
> rs-wc - A performant wc-like utility in Rust. Counts lines, words, bytes, and characters in files or stdin.

## Features

- Count lines, words, bytes, and characters in files or standard input.
- Supports multiple files with aggregated totals.
- High performance and memory efficiency.
- Cross-platform compatibility.

## Installation

1. Ensure you have [Rust](https://www.rust-lang.org/) installed.
2. Clone the repository:
    ```bash
    git clone https://github.com/chineduCoded/rs-wc.git
    cd rs-wc
    ```
3. Build the project:
    ```bash
    cargo build --release
    ```
4. Run the binary:
    ```bash
    ./target/release/rs-wc
    ```

## Usage

Basic usage:
```bash
rs-wc <file1> <file2> ...
```

Example:
```bash
rs-wc -l file.txt
```

Using stdin:
```bash
echo "Hello, world!" | rs-wc
```

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request.

<!-- ## License -->

<!-- This project is licensed under the [MIT License](LICENSE). -->