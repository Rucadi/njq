use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;



#[test]
fn test_missing_nix_expr() {
    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Missing <nix_expr>."));
}

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
        .write_stdin(r#"{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"["Alice","Bob"]"#));
}

#[test]
fn test_file_json() {
    // Create a temporary JSON file
    let mut file = tempfile::NamedTempFile::new().unwrap();
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
fn test_escaped_mode() {
    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["--escaped", "input"])
        .write_stdin(r#""Hello\nWorld""#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""\"Hello\\nWorld\"""#));
}