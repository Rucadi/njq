use std::env;
use std::io;
use std::io::Read;
use std::process;
use snix_eval::SourceCode;
use snix_eval::{Evaluation, Value};
use snix_eval::builtin_macros::builtins;
use serde_json::Value as JsonValue;

#[builtins]
mod custom_builtins {
    use embed_nu::nu_protocol;
    use snix_eval::generators::{Gen, GenCo};
    use snix_eval::{ErrorKind, NixString, Value};
    use bstr::ByteSlice;

    #[builtin("nushell")]
    pub async fn builtin_prepend_hello(co: GenCo, x: Value) -> Result<Value, ErrorKind> {
        match x {
            Value::String(s) => {
                let mut ctx = embed_nu::Context::builder()
                    .with_command_groups(embed_nu::CommandGroupConfig::default().all_groups(true))
                    .unwrap()
                    .add_parent_env_vars()
                    .build()
                    .unwrap();
    
                let pipeline = ctx
                    .eval_raw(s.to_str().unwrap(), embed_nu::PipelineData::empty())
                    .unwrap();
    
                let result = pipeline.into_value(nu_protocol::Span::unknown()).unwrap();
    
                let output_string = match result {
                    nu_protocol::Value::Int { val, .. } => val.to_string(),
                    nu_protocol::Value::Float { val, .. } => val.to_string(),
                    nu_protocol::Value::Bool { val, .. } => val.to_string(),
                    nu_protocol::Value::String { val, .. } => val,
                    nu_protocol::Value::List { vals, .. } => vals
                        .into_iter()
                        .map(|v| v.into_string().unwrap_or("<err>".into()))
                        .collect::<Vec<_>>()
                        .join(", "),
                    other => other.into_string().unwrap_or("<unknown>".into()),
                };
    
                Ok(Value::String(NixString::from(output_string)))
            }
            _ => Err(ErrorKind::TypeError {
                expected: "string",
                actual: "not string",
            }),
        }
    }
}



fn print_usage(prog: &str) {
    eprintln!("Usage: {} [--escaped] [--nix] [--indent] <nix_expr> [json_file]", prog);
    eprintln!("  --escaped        Print output with JSON escapes");
    eprintln!("  --nix            Treat <nix_expr> as a self-contained expression (skip JSON input)");
    eprintln!("  --indent         Pretty-print JSON output with 2-space indentation");
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

pub fn test_fn(s: &str) -> String {
    format!("Called Rust with: {}", s)
}


fn printEvalErrors(result: &snix_eval::EvaluationResult, source_map: &snix_eval::SourceCode )
{
    if !result.errors.is_empty() {
        // Handle and display all errors
        for error in &result.errors {
            eprintln!("Error: {}", error);
        }
    }
    
    if !result.warnings.is_empty() {
        // Handle and display all warnings (optional)
        for warning in &result.warnings {
            eprintln!("Warning: {}", warning.fancy_format_str(&source_map));
        }
    }

}
fn evaluate_to_value(code: &str) -> Option<Value> {
    let cwd = env::current_dir().unwrap_or_else(|_| "/".into()).to_string_lossy().into_owned();
    let evaluator = Evaluation::builder_impure().build();
    let source_map = evaluator.source_map();
    let result = evaluator.evaluate(code, Some(cwd.into()));
    
    printEvalErrors(&result, &source_map);

    result.value
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prog = args.get(0).unwrap_or(&"program".to_string()).clone();
    let mut nix_only = false;
    let mut pretty = false;
    let mut positional = Vec::new();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--nix"     => nix_only = true,
            "--pretty"  => pretty = true,
            "help" | "--help" | "-h" => print_usage(&prog),
            _ => positional.push(arg.clone()),
        }
    }

    if positional.is_empty() {
        eprintln!("Error: Missing <nix_expr>.");
        print_usage(&prog);
    }

    let code_expr = &positional[0];
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
        // Read raw JSON from stdin and re-serialize to ensure valid quoting
        let raw_json = slurp_stdin().unwrap_or_else(|e| { eprintln!("Error reading stdin: {}", e); process::exit(1); });
        // Validate and quote via serde_json
        let js_val: JsonValue = serde_json::from_str(&raw_json)
            .unwrap_or_else(|e| { eprintln!("Invalid JSON input: {}", e); process::exit(1); });
        let compact = serde_json::to_string(&js_val).unwrap();
        format!("builtins.fromJSON ('{}')", compact)
    };

    // Build the Nix expression to JSON
    let full_code: String = format!("with builtins; toJSON ({})", code_expr);

    let input_val = evaluate_to_value(&input_expr)
        .unwrap_or_else(|| { eprintln!("Evaluation of input_expr failed."); process::exit(1); });

    let builder = Evaluation::builder_impure()
        .add_builtins([("input", input_val)])
        .add_builtins(custom_builtins::builtins());
    let evaluator = builder.build();
    let cwd = env::current_dir().unwrap_or_else(|_| "/".into()).to_string_lossy().into_owned();
    let source_map = evaluator.source_map();
    let result = evaluator.evaluate(&full_code, Some(cwd.into()));
    printEvalErrors(&result, &source_map);


    let out_str = result.value
        .map(|v| v.to_string())
        .unwrap_or_else(|| { eprintln!("Evaluation failed or returned null."); process::exit(1); });

    let out_str: String = serde_json::from_str(&out_str).unwrap();
    let json_val: JsonValue = serde_json::from_str(&out_str)
            .unwrap_or_else(|e| { eprintln!("Internal JSON parse error: {}", e); process::exit(1) });
    if pretty {
        println!("{}", serde_json::to_string_pretty(&json_val).unwrap());
    } else {
        // just print the compact text directly
        println!("{}", out_str);
    } 
}
#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_evaluate_to_value() {
        // Simple Nix expression
        let result = evaluate_to_value("42");
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), "42");

        // Invalid Nix expression
        let result = evaluate_to_value("invalid + syntax");
        assert!(result.is_none());
    }
}