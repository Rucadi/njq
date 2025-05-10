use clap::Parser;
use colored_json::ToColoredJson;
use serde_json::Value as JsonValue;
use snix_eval::{Evaluation, Value};
use std::env;
use std::io::IsTerminal;
use std::io::{self, Read};
use std::process;
use std::str;
/// A small CLI to evaluate Nix expressions with optional JSON input
#[derive(Parser, Debug)]
#[clap(
    name = "njq",
    version,
    author,
    about = "A small CLI to query json or nix dictionaries using Nix Expressions ",
    override_usage = "njq [OPTIONS] <EXPR> [FILE]",
    after_help = "\
    The input file (or stdin) should contain:\n\
    - JSON data, if --nix is not provided.\n\
    - A Nix expression that evaluates to an attribute set, if --nix is provided.\n\n\
    EXAMPLES:\n\
        # Evaluate a Nix expression with JSON input from stdin\n\
        echo '{\"key\": \"value\"}' | njq 'input.key'\n\n\
        # Evaluate a Nix expression with JSON input from a file\n\
        njq 'input.key' input.json\n\n\
        # Evaluate a Nix expression with another Nix expression as input\n\
        njq --nix 'input.attr' nix_file.nix"
)]
struct Opt {
    /// Evaluate a Nix expression from a file or stdin
    #[clap(long = "nix", short = 'n', value_name = "NIX_EXPR")]
    nix: bool,

    /// Pretty-print JSON output with 2-space indentation
    #[clap(long)]
    compact: bool,

    /// The Nix expression to evaluate (quoted)
    #[clap(value_name = "EXPR")]
    query_expression: Option<String>,

    /// Path to JSON input file; if omitted, reads from stdin
    #[clap(value_name = "JSON_OR_NIX_FILE")]
    file_path: Option<String>,
}

fn nix_attr_set_from_json(s: &str) -> Option<Value> {
    serde_json::from_str(s).ok()
}

fn main() {
    let opt = Opt::parse();

    let query_expression = opt.query_expression.as_deref().unwrap_or_else(|| {
        if !std::io::stdin().is_terminal() {
            "input"
        } else {
            let mut cmd = <Opt as clap::CommandFactory>::command();
            cmd.print_help().unwrap();
            process::exit(0);
        }
    });

    let file_content = if opt.file_path.is_some() {
        let path = opt.file_path.unwrap();
        std::fs::read_to_string(path).unwrap_or_else(|_| {
            exit_err("failed to read file, unexistent or invalid path");
        })
    } else {
        let mut raw = String::new();
        io::stdin().read_to_string(&mut raw).unwrap();
        raw
    };

    let attr_set = if opt.nix {
        evaluate_with_input(&file_content, Value::Null)
            .unwrap_or_else(|| exit_err("evaluation failed"))
    } else {
        nix_attr_set_from_json(&file_content)
            .unwrap_or_else(|| exit_err("failed to parse JSON input"))
    };

    let full_code = format!("with builtins; toJSON ({})", query_expression);
    let result =
        evaluate_with_input(&full_code, attr_set).unwrap_or_else(|| exit_err("evaluation failed"));
    let raw = result.to_string().replace("\\$", "$");
    let json_value: JsonValue = serde_json::from_str(&raw)
        .unwrap_or_else(|e| exit_err(&format!("invalid JSON output: {}", e)));

    let json_value_final = serde_json::from_str(&json_value.as_str().unwrap())
        .map_err(|e| format!("Invalid JSON output: {}", e))
        .unwrap();

    print_output(&json_value_final, opt.compact);
}

/// Evaluate main expression with `input` bound and return the result
fn evaluate_with_input(code: &str, input: Value) -> Option<Value> {
    let evaluator = Evaluation::builder_impure()
        .add_builtins([("input", input)])
        .build();
    let source_map = evaluator.source_map();
    let cwd = env::current_dir().unwrap_or_else(|_| "/".into());
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

fn print_output(json: &JsonValue, compact: bool) {
    match json {
        JsonValue::String(s) => println!("{}", s),
        _ => {
            if compact {
                println!("{}", json);
                return;
            }

            let json_string = serde_json::to_string_pretty(json);
            match json_string {
                Ok(s) => {
                    match s.to_colored_json_auto() {
                        Ok(colored) => println!("{}", colored),
                        Err(_) => println!("{}", s), // Fallback to plain JSON string
                    }
                }
                Err(e) => eprintln!("Failed to serialize JSON: {}", e),
            }
        }
    }
}

/// Print an error and exit
fn exit_err(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    process::exit(1);
}
