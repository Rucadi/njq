use std::env;
use std::io;
use std::io::Read;
use std::process;
use tvix_eval::{Evaluation, Value};

fn print_usage(prog: &str) {
    eprintln!("Usage: {} [--escaped] [--nix] <nix_expr> [json_file]", prog);
    eprintln!("  --escaped        Print output with JSON escapes");
    eprintln!("  --nix        Treat <nix_expr> as a self-contained expression (skip JSON input)");
    eprintln!("  <nix_expr>   The Nix expression to evaluate (quoted)");
    eprintln!("  [json_file]  Path to JSON input file; if omitted, reads from stdin");
    eprintln!("  help         Show this help message");
    process::exit(1);
}

fn slurp_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn nix_string_literal(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn unescape_json(s: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some(next) => match next {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'b' => out.push('\x08'),
                    'f' => out.push('\x0C'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let mut code = 0;
                        for _ in 0..4 {
                            match chars.next() {
                                Some(h) if h.is_digit(16) => {
                                    code = code * 16 + h.to_digit(16).unwrap();
                                }
                                _ => return Err("Invalid unicode escape".to_string()),
                            }
                        }
                        match char::from_u32(code) {
                            Some(ch) => out.push(ch),
                            None => return Err("Invalid unicode codepoint".to_string()),
                        }
                    }
                    _ => out.push(next),
                },
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

fn evaluate_to_value(code: &str) -> Option<Value> {
    let cwd = env::current_dir().unwrap_or_else(|_| "/".into()).to_string_lossy().into_owned();
    let evaluator = Evaluation::builder_impure().build();
    let result = evaluator.evaluate(code, Some(cwd.into()));
    result.value
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prog = args.get(0).unwrap_or(&"program".to_string()).clone();
    let mut raw = true;
    let mut nix_only = false;
    let mut positional = Vec::new();
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--escaped" => raw = false,
            "--nix" => nix_only = true,
            "help" | "--help" | "-h" => print_usage(&prog),
            _ => positional.push(arg.clone()),
        }
    }
    if positional.is_empty() {
        eprintln!("Error: Missing <nix_expr>.");
        print_usage(&prog);
    }
    let code_expr = positional[0].clone();
    let file_path = if positional.len() > 1 {
        Some(positional[1].clone())
    } else {
        None
    };
    let input_expr = if nix_only {
        "null".to_string()
    } else if let Some(file_path) = &file_path {
        let nix_path = file_path.replace('\\', "/");
        let nix_path = if nix_path.starts_with('/') || nix_path.starts_with("./") {
            nix_path
        } else {
            format!("./{}", nix_path)
        };
        format!("builtins.fromJSON (builtins.readFile {})", nix_path)
    } else {
        let json = match slurp_stdin() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                process::exit(1);
            }
        };
        let nix_json = nix_string_literal(&json);
        format!("builtins.fromJSON ({})", nix_json)
    };
    let full_code = format!("with builtins; toJSON ({})", code_expr);
    let input_val = match evaluate_to_value(&input_expr) {
        Some(val) => val,
        None => {
            eprintln!("Evaluation of input_expr failed.");
            process::exit(1);
        }
    };
    // print the nix expressionbs

    let builder = Evaluation::builder_impure().add_builtins([("input", input_val)]);
    let evaluator = builder.build();
    let cwd = env::current_dir().unwrap_or_else(|_| "/".into()).to_string_lossy().into_owned();
    let result = evaluator.evaluate(&full_code, Some(cwd.into()));
    let out_str = match result.value {
        Some(v) => v.to_string(),
        None => {
            eprintln!("Evaluation failed or returned null.");
            process::exit(1);
        }
    };
    if raw {
        let stripped = if out_str.starts_with('"') && out_str.ends_with('"') {
            &out_str[1..out_str.len() - 1]
        } else {
            &out_str
        };
        match unescape_json(stripped) {
            Ok(unescaped) => println!("{}", unescaped),
            Err(e) => {
                eprintln!("Error unescaping JSON: {}", e);
                process::exit(1);
            }
        }
    } else {
        println!("{}", out_str);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nix_string_literal() {
        assert_eq!(nix_string_literal("hello"), "\"hello\"");
        assert_eq!(nix_string_literal("hello\nworld"), "\"hello\\nworld\"");
        assert_eq!(nix_string_literal("quote\"backslash\\"), "\"quote\\\"backslash\\\\\"");
        assert_eq!(nix_string_literal("\r\t"), "\"\\r\\t\"");
    }

    #[test]
    fn test_unescape_json() {
        assert_eq!(unescape_json("hello").unwrap(), "hello");
        assert_eq!(unescape_json("hello\\nworld").unwrap(), "hello\nworld");
        assert_eq!(unescape_json("quote\\\"backslash\\\\").unwrap(), "quote\"backslash\\");
        assert_eq!(unescape_json("\\u0041").unwrap(), "A");
        assert_eq!(unescape_json("\\u00E9").unwrap(), "é");
        assert!(unescape_json("\\uGGGG").is_err());
        assert!(unescape_json("\\uD800").is_err()); // Invalid surrogate
    }

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