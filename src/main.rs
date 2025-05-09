use std::fs;
use std::io::{self, Read, Write};
use std::process;
use snix_eval::{Evaluation, Value};
use serde_json::Value as JsonValue;
use tempfile::NamedTempFile;
use clap::{Parser, ArgGroup};
use std::env;

/// A small CLI to evaluate Nix expressions with optional JSON input
#[derive(Parser, Debug)]
#[clap(
    name = "snix-eval",
    version,
    author,
    about,
    override_usage = "snix-eval [--nix | --nix-file FILE] [--pretty] <EXPR | --nix-file FILE> [JSON_FILE]",
    after_help = "Positional JSON_FILE is required unless --nix or --nix-file is used.\n\
If omitted, JSON is read from stdin.\n\
The 'input' binding in the Nix expression refers to the parsed JSON object."
)]
#[clap(group(
    ArgGroup::new("expr_source")
        .required(true)
        .args(&["expr", "nix_file"])
))]
struct Opt {
    /// Treat <expr> as a self-contained Nix expression (skip JSON input)
    #[clap(long)]
    nix: bool,

    /// Read the Nix expression directly from file
    #[clap(long = "nix-file", value_name = "FILE")]
    nix_file: Option<String>,

    /// Pretty-print JSON output with 2-space indentation
    #[clap(long)]
    pretty: bool,

    /// The Nix expression to evaluate (quoted)
    #[clap(value_name = "EXPR")]
    expr: Option<String>,

    /// Path to JSON input file; if omitted, reads from stdin
    #[clap(value_name = "JSON_FILE")]
    json_file: Option<String>,
}

fn main() {
    let opt = Opt::parse();

    // Load Nix code from file or CLI argument
    let code_expr = match &opt.nix_file {
        Some(path) => read_file_or_exit(path, "nix expression"),
        None => opt.expr.clone().unwrap(),
    };

    // Input expression setup
    let (input_expr, _temp_file): (String, Option<NamedTempFile>) = if opt.nix || opt.nix_file.is_some() {
        ("null".to_string(), None)
    } else if let Some(json_path) = &opt.json_file {
        let path = normalize_path(json_path);
        (format!("builtins.fromJSON (builtins.readFile \"{}\")", path), None)
    } else {
        let (path, tmp) = write_temp_json().unwrap_or_else(|e| exit_err(&format!("Failed to prepare JSON: {}", e)));
        let path = pathdiff::diff_paths(path, env::current_dir().unwrap()).unwrap();
        let path = path.to_string_lossy();
        let path = normalize_path(&path);
        (format!("builtins.fromJSON (builtins.readFile ({}))", path), Some(tmp))
    };

    println!("{}", input_expr);
    // Evaluate input JSON to a Nix value
    let input_val = evaluate_to_value(&input_expr).unwrap_or_else(|| exit_err("input JSON evaluation failed"));

    // Evaluate the Nix expression with the input
    let full_code = format!("with builtins; toJSON ({})", code_expr);
    let result = evaluate_with_input(&full_code, input_val);

    // Output
    match result {
        Ok(json_str) => print_output(&json_str, opt.pretty),
        Err(msg) => exit_err(&msg),
    }
}

/// Read a file into a string or exit with an error message
fn read_file_or_exit(path: &str, desc: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| exit_err(&format!("Failed to read {} '{}': {}", desc, path, e)))
}

/// Normalize file path for Nix (make it absolute or prefixed with ./)
fn normalize_path(path: &str) -> String {
    let p = path.replace('\\', "/");
    if p.starts_with('/') || p.starts_with("./") {
        p
    } else {
        format!("{}", p)
    }
}

/// Read JSON from stdin, compact it, write to a temp file, and return its path and handle
fn write_temp_json() -> io::Result<(String, NamedTempFile)> {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw)?;
    let js: JsonValue = serde_json::from_str(&raw)?;
    let compact = serde_json::to_string(&js)?;
    let mut tmp = NamedTempFile::new()?;
    write!(tmp, "{}", compact)?;
    let path = tmp.path().to_string_lossy().into_owned();
    Ok((path, tmp))
}

/// Evaluate a Nix expression to a JSON-like Value, printing errors/warnings
fn evaluate_to_value(code: &str) -> Option<Value> {
    let evaluator = Evaluation::builder_impure().build();
    let source_map = evaluator.source_map();
    let cwd = std::env::current_dir().unwrap_or_else(|_| "/".into());
    let res = evaluator.evaluate(code, Some(cwd));
    for err in &res.errors {
        eprintln!("Error: {}", err);
    }
    for warn in &res.warnings {
        eprintln!("Warning: {}", warn.fancy_format_str(&source_map));
    }
    res.value
}

/// Evaluate main expression with `input` bound and return JSON string
fn evaluate_with_input(code: &str, input: Value) -> Result<String, String> {
    let evaluator = Evaluation::builder_impure()
        .add_builtins([("input", input)])
        .build();
    let source_map = evaluator.source_map();
    let cwd = std::env::current_dir().unwrap_or_else(|_| "/".into());
    let res = evaluator.evaluate(code, Some(cwd));
    for err in &res.errors {
        eprintln!("Error: {}", err);
    }
    for warn in &res.warnings {
        eprintln!("Warning: {}", warn.fancy_format_str(&source_map));
    }
    let raw = res.value.map(|v| v.to_string()).ok_or_else(|| "evaluation returned null".to_string())?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid JSON output: {}", e))
}

/// Print the final JSON result (pretty or compact)
fn print_output(json: &str, pretty: bool) {
    if pretty {
        let v: JsonValue = serde_json::from_str(json).unwrap();
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else {
        println!("{}", json);
    }
}

/// Print an error and exit
fn exit_err(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    process::exit(1);
}
