use std::env;
use std::io;
use std::io::Read;
use std::process;
use snix_eval::{Evaluation, Value};
use serde_json::Value as JsonValue;


fn print_usage(prog: &str) {
    eprintln!("Usage: {} [--nix] [--pretty] <nix_expr> [json_file]", prog);
    eprintln!("  --nix            Treat <nix_expr> as a self-contained expression (skip JSON input)");
    eprintln!("  --pretty         Pretty-print JSON output with 2-space indentation");
    eprintln!("  <nix_expr>       The Nix expression to evaluate (quoted)");
    eprintln!("  [json_file]      Path to JSON input file; if omitted, reads from stdin");
    eprintln!("  help             Show this help message");
    process::exit(1);
}

fn slurp_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn print_eval_errors(result: &snix_eval::EvaluationResult, source_map: &snix_eval::SourceCode) {
    if !result.errors.is_empty() {
        for error in &result.errors {
            eprintln!("Error: {}", error);
        }
    }
    if !result.warnings.is_empty() {
        for warning in &result.warnings {
            eprintln!("Warning: {}", warning.fancy_format_str(source_map));
        }
    }
}

fn evaluate_to_value(code: &str) -> Option<Value> {
    let cwd = env::current_dir()
        .unwrap_or_else(|_| "/".into())
        .to_string_lossy()
        .into_owned();
    let evaluator = Evaluation::builder_impure().build();
    let source_map = evaluator.source_map();
    let result = evaluator.evaluate(code, Some(cwd.into()));
    
    print_eval_errors(&result, &source_map);
    result.value
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prog = args.get(0).unwrap_or(&"program".to_string()).clone();
    let mut nix_only = false;
    let mut pretty = false;
    let mut positional = Vec::with_capacity(2); // Typically 1 or 2 positional args

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--nix" => nix_only = true,
            "--pretty" => pretty = true,
            "help" | "--help" | "-h" => print_usage(&prog),
            _ => positional.push(arg.clone()),
        }
    }

    if positional.is_empty() {
        eprintln!("Error: Missing <nix_expr>.");
        print_usage(&prog);
    }

    let file_path = positional.get(1).cloned();

    // Prepare the JSON input expression for Nix
    let input_expr = if nix_only {
        "null".to_string()
    } else if let Some(fp) = file_path {
        let nix_path = fp.replace('\\', "/");
        let nix_path = if nix_path.starts_with('/') || nix_path.starts_with("./") {
            nix_path
        } else {
            format!("./{}", nix_path)
        };
        format!("builtins.fromJSON (builtins.readFile {})", nix_path)
    } else {
        let raw_json = slurp_stdin().unwrap_or_else(|e| {
            eprintln!("Error reading stdin: {}", e);
            process::exit(1);
        });
        let js_val: JsonValue = serde_json::from_str(&raw_json).unwrap_or_else(|e| {
            eprintln!("Invalid JSON input: {}", e);
            process::exit(1);
        });
        let compact = serde_json::to_string(&js_val).unwrap();
        format!("builtins.fromJSON (''{}'')", compact.replace("''", "'''"))
    };

    // Build the Nix expression to JSON, inlining code_expr
    let full_code = format!("with builtins; toJSON ({})", &positional[0]);

    let input_val = evaluate_to_value(&input_expr).unwrap_or_else(|| {
        eprintln!("Evaluation of input json failed.");
        process::exit(1);
    });

    let builder = Evaluation::builder_impure()
        .add_builtins([("input", input_val)]);
    let evaluator = builder.build();
    let cwd = env::current_dir()
        .unwrap_or_else(|_| "/".into())
        .to_string_lossy()
        .into_owned();
    let source_map = evaluator.source_map();
    let result = evaluator.evaluate(&full_code, Some(cwd.into()));
    print_eval_errors(&result, &source_map);

    let out_str = result.value.map(|v| v.to_string()).unwrap_or_else(|| {
        eprintln!("Evaluation failed or returned null.");
        process::exit(1);
    });
    let out_str: String = serde_json::from_str(&out_str).unwrap();


    if pretty {
        let json_val: JsonValue = serde_json::from_str(&out_str).unwrap_or_else(|e| {
            eprintln!("Invalid JSON output: {}", e);
            process::exit(1);
        });
        println!("{}", serde_json::to_string_pretty(&json_val).unwrap());
    } else {
        println!("{}", out_str);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_to_value() {
        let result = evaluate_to_value("42");
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), "42");

        let result = evaluate_to_value("invalid + syntax");
        assert!(result.is_none());
    }
}