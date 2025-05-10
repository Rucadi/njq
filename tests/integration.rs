use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::write;
use std::io::Write;
use tempfile::NamedTempFile;



#[test]
fn test_normal_mode_json_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{"numbers": [1, 2, 3], "greeting": "hello"}}"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["input.numbers", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[
  1,
  2,
  3
]"));
}


#[test]
fn test_normal_mode_json_file_compact() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{"numbers": [1, 2, 3], "greeting": "hello"}}"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["input.numbers", file.path().to_str().unwrap(), "--compact"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[1,2,3]"));
}


#[test]
fn test_normal_mode_json_file_dollar() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{"numbers": [1, 2, 3], "greeting": "he${{hooho}}llo"}}"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["input.greeting", file.path().to_str().unwrap(), "--compact"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("he${hooho}llo"));
}

#[test]
fn test_normal_mode_json_stdin() {
    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["input.numbers"])
        .write_stdin(r#"{"numbers": [1, 2, 3]}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[
  1,
  2,
  3
]"));
}


#[test]
fn test_normal_mode_nix_file() {
    let nix_file = NamedTempFile::new().unwrap();
    write(nix_file.path(), "[1 2 3]").unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["input", "--nix", nix_file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[
  1,
  2,
  3
]"));
}

#[test]
fn test_normal_mode_nix_file_with_expr() {
    let nix_file = NamedTempFile::new().unwrap();
    write(nix_file.path(), "[1 2 3]").unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&[
        "map (x: x * 2) input",
        "--nix",
        nix_file.path().to_str().unwrap(),
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[
  2,
  4,
  6
]"));
}

#[test]
fn test_normal_mode_invalid_expr() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"{{"key": "value"}}"#).unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["invalid", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("evaluation failed"));
}

#[test]
fn test_normal_mode_missing_json_file() {
    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["input", "nonexistent.json"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("failed to read file"));
}

#[test]
fn test_normal_mode_invalid_json_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"{{key: value}}"#).unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["input", file.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: failed to parse JSON input\n"));
}

#[test]
fn test_normal_mode_invalid_json_stdin() {
    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["input"])
        .write_stdin(r#"{{key: value}}"#);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: failed to parse JSON input\n"));
}

#[test]
fn test_normal_mode_string_output() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{"numbers": [1, 2, 3], "greeting": "hello"}}"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&[r#"input.greeting"#, file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_normal_mode_json_string_output() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{"numbers": [1, 2, 3], "greeting": "hello"}}"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&[
        r#"builtins.toJSON { a = 1; }"#, 
        file.path().to_str().unwrap(),
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"{"a":1}"#));
}

#[test]
fn test_normal_mode_number_output() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{"numbers": [1, 2, 3], "greeting": "hello"}}"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["42", file.path().to_str().unwrap()]).unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["42", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_normal_mode_boolean_output() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{"numbers": [1, 2, 3], "greeting": "hello"}}"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["true", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_normal_mode_null_output() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{"numbers": [1, 2, 3], "greeting": "hello"}}"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("njq").unwrap();
    cmd.args(&["null", file.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("null"));
}