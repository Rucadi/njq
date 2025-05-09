use clap::{ArgGroup, Parser};
use serde_json::Value as JsonValue;
use snix_eval::{Evaluation, Value};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process;
use std::str;
use tempfile::NamedTempFile;

/// A small CLI to evaluate Nix expressions with optional JSON input
#[derive(Parser, Debug)]
#[clap(
    name = "njq",
    version,
    author,
    about,
    override_usage = "njq[--nix | --nix-file FILE] [--pretty] <EXPR | --nix-file FILE> [JSON_FILE]",
    after_help = "Positional JSON_FILE is required unless --nix or --nix-file is used.\n\
If omitted, JSON is read from stdin.\n\
The 'input' binding in the Nix expression refers to the parsed JSON object."
)]
struct Opt {
    /// The file or stdin is treated as a Nix expression to evaluate or JSON to print
    #[clap(long = "no-expr", short = 'p')]
    no_expr: bool,

    /// Evaluate a Nix expression from a file or stdin
    #[clap(long = "nix", short = 'n', value_name = "NIX_EXPR")]
    nix: bool,

    // Evaluate a nix expression directly from this attribute, not compatible with any other options
    #[clap(long = "nix-eval", short = 'E', value_name = "EVAL")]
    nix_eval: Option<String>,

    /// Pretty-print JSON output with 2-space indentation
    #[clap(long)]
    pretty: bool,

    /// The Nix expression to evaluate (quoted)
    #[clap(value_name = "EXPR")]
    expr: Option<String>,

    /// Path to JSON input file; if omitted, reads from stdin
    #[clap(value_name = "JSON_OR_NIX_FILE")]
    file_path: Option<String>,
}

fn main() {
    let opt = Opt::parse();

    // if --no-expr is used then EXPR is JSON_OR_NIX_FILE since it's a positional argument. 
    // If this is algo not set, then we assume we get it from stdin.

    // if --nix-eval then we only expect EXPR and not JSON_FIULE not stdin

    if opt.nix_eval.is_some()  {
        let result = evaluate_with_input(&opt.nix_eval.unwrap(), Value::Null)
            .unwrap_or_else(|| exit_err("input JSON evaluation failed"));
        let string_result = result.to_string();
        println!("{}", string_result);
        return;
    }


    // no-expr cannot be used with --nix-eval 
    if opt.no_expr && opt.nix_eval.is_some() {
        exit_err("Cannot use --no-expr with --nix-eval");
    }
    
    if !opt.no_expr && opt.expr.is_none() {
        exit_err("No expression provided. Use --no-expr.");
    }
    

    //if --no-expr is set then json file is in EXPR, get thet path, if it's not in EXPR then assume it's stdin 
    //else get it from json_file
// Determine where to read JSON from:
    let (file_path, _temp_file): (String, Option<NamedTempFile>) = if opt.no_expr {
        // In --no-expr mode, EXPR is actually the JSON file path (if provided)
        if let Some(expr_path) = &opt.expr {
            (normalize_path(expr_path), None)
        } else {
            // No positional, slurp stdin → tempfile
            let (tmp_path, tmp) = write_temp_json()
                .unwrap_or_else(|e| exit_err(&format!("failed to read JSON from stdin: {}", e)));
            (normalize_path(&tmp_path), Some(tmp))
        }
    } else {
        // Normal mode: JSON file comes from --json-file (or again, stdin → tempfile)
        if let Some(json_path) = &opt.file_path {
            (normalize_path(json_path), None)
        } else {
            let (tmp_path, tmp) = write_temp_json()
                .unwrap_or_else(|e| exit_err(&format!("failed to read JSON from stdin: {}", e)));
            (normalize_path(&tmp_path), Some(tmp))
        }
    };

    let evaluated_input_expr = if opt.no_expr {
        Value::Null
    } else {
        let input_expr = if opt.nix {
            format!("import {}", file_path)
        } else {
            format!("builtins.fromJSON (builtins.readFile {})", file_path)
        };
        println!("EXPR: {}", input_expr);
        evaluate_with_input(&input_expr, Value::Null)
            .unwrap_or_else(|| exit_err("input JSON evaluation failed"))
    };
    


    let code_expr = if opt.no_expr {
        "input"
    } else {
       opt.expr.as_deref().unwrap_or_else(|| {
            exit_err("No Nix expression provided. Use --nix or --nix-eval.")
        })
    };

    // Evaluate the Nix expression with the input
    let full_code = format!("with builtins; toJSON ({})", code_expr);


    let result = evaluate_with_input(&full_code, evaluated_input_expr);

    let raw = result
        .map(|v| v.to_string())
        .ok_or_else(|| "evaluation returned null".to_string())
        .unwrap();
    let result2 = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid JSON output: {}", e))
        .unwrap();

    print_output(&result2, opt.pretty)
}

/// Normalize file path for Nix (make it absolute or prefixed with ./)
fn normalize_path(path: &str) -> String {
    let path = pathdiff::diff_paths(path, env::current_dir().unwrap()).unwrap();
    let path = path.to_string_lossy();

    let p = path.replace('\\', "/");
    if p.starts_with('/') || p.starts_with("./") || p.starts_with("../") {
        p
    } else {
        format!("./{}", p)
    }
}

/// Read JSON from stdin, compact it, write to a temp file, and return its path and handle
fn write_temp_json() -> io::Result<(String, NamedTempFile)> {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw)?;

    let mut tmp = NamedTempFile::new()?;
    write!(tmp, "{}", String::from(&raw))?;
    let path = tmp.path().to_string_lossy().into_owned();
    Ok((path, tmp))
}

/// Evaluate main expression with `input` bound and return JSON string
fn evaluate_with_input(code: &str, input: Value) -> Option<Value> {
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
    res.value
}

/// Print the final JSON result (pretty or compact)
fn print_output(json: &JsonValue, pretty: bool) {
    match json {
        // if the result is a JSON string, print it *without* the surrounding quotes or backslashes
        JsonValue::String(s) => {
            println!("{}", s);
        }

        // otherwise print whole JSON as normal
        _ if pretty => {
            println!("{}", serde_json::to_string_pretty(json).unwrap());
        }
        _ => {
            println!("{}", json);
        }
    }
}

/// Print an error and exit
fn exit_err(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    process::exit(1);
}
