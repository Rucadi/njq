use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use std::fs::write;
use tempfile::NamedTempFile;

#[test]
fn test_nix_mode() {
    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["--nix", "builtins.length [1 2 3 4]"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_stdin_json() {
    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["map (u: u.name) input.users"])
        .write_stdin(r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"["Alice","Bob"]"#));
}

#[test]
fn test_file_json() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{ "users": [{{ "name": "Alice", "age": 30 }}, {{ "name": "Bob", "age": 25 }}] }}"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["filter (u: u.age > 27) input.users", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"[{"age":30,"name":"Alice"}]"#));
}

#[test]
fn test_nix_file_expr() {
    let expr = "map (x: x * 2) input.values";
    let file = NamedTempFile::new().unwrap();
    write(file.path(), expr).unwrap();

    let mut data = NamedTempFile::new().unwrap();
    writeln!(data, r#"{{"values": [1, 2, 3]}}"#).unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["--nix-file", file.path().to_str().unwrap(), data.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[2,4,6]"));
}

#[test]
fn test_pretty_output() {
    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["--pretty", "input"])
        .write_stdin(r#"{"hello":"world","x":[1,2]}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("{\n  \"hello\": \"world\""));
}

#[test]
fn test_missing_expr_error() {
    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("USAGE"));
}

#[test]
fn test_invalid_json_input() {
    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["input.foo"])
        .write_stdin(r#"{ invalid json "#);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to prepare JSON"));
}

#[test]
fn test_null_json_when_nix_file() {
    let expr = "input == null";
    let mut file = NamedTempFile::new().unwrap();
    write(file.path(), expr).unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["--nix-file", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("true"));
}
