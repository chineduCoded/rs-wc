#[cfg(test)]
mod integration_tests {
    type WcResult<T> = Result<T, Box<dyn std::error::Error>>;
    use assert_cmd::Command;
    use predicates as predicate;

    #[test]
    fn test_cli_with_file() -> WcResult<()> {
        let mut cmd = Command::cargo_bin("rs-wc")?;
        let assert = cmd.arg("Cargo.toml").assert();
        assert.success().stdout(predicate::str::contains("Cargo.toml"));
        Ok(())
    }

    #[test]
    fn test_cli_stdin() -> WcResult<()> {
        let mut cmd = Command::cargo_bin("rs-wc")?;
        let assert = cmd.write_stdin("test input").assert();
        assert.success().stdout(predicate::str::contains("1"));
        Ok(())
    }

    #[test]
    fn test_cli_json_output() -> WcResult<()> {
        let mut cmd = Command::cargo_bin("rs-wc")?;
        let assert = cmd.args(&["-f", "json", "Cargo.toml"]).assert();
        assert.success().stdout(predicate::str::is_match(r#""filename": "Cargo.toml""#)?);
        Ok(())
    }
}