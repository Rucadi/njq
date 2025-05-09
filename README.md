
# njq

A small CLI to query JSON or Nix data using Nix expressions.

## Description

`njq` (Nix JQ) lets you use Nix as a lightweight query language for JSON data (or nix expressions). 
It parses JSON into a Nix "builtin" named `input`, evaluates your Nix expression, and prints the result of it.

It uses [Snix](https://snix.dev/) as nix implementation, altough slightly modified fork to support windows.

All the heavy-weight is done by Snix, njq interfaces Snix to provide this functionality.

You can check the available builtins for nix here: https://nix.dev/manual/nix/2.24/language/builtins.html


## Installation

```bash

# build locally
git clone https://github.com/rucadi/njq.git
cd njq
cargo build --release
# The binary will be at target/release/njq

download it from release section on github


use nix:

nix profile install github:rucadi/njq
```

## Usage

```
njq [OPTIONS] <EXPR> [FILE]
```

- `EXPR`: The Nix expression to evaluate. This expression can use `input` to refer to the provided input data.
- `FILE`: Path to the input file. If not provided, reads from stdin.

## Options

- `--nix`, `-n`: Read nix files instead of json
- `--compact`: Output JSON in compact format instead of pretty-printed. If the result is a string, it is always printed without quotes.

## Input

The input can be:

- JSON data, if `--nix` is not provided. The JSON is parsed into a corresponding Nix value (e.g., object to attribute set, array to list, etc.) and bound to `input` in the Nix expression.
- A Nix expression, if `--nix` is provided. T

If a file path is provided, it reads from that file; otherwise, it reads from stdin.

## Output

The output is the result of evaluating the Nix expression, converted to JSON:

- If the result is a string, it is printed directly without quotes.
- For other types (numbers, booleans, lists, attribute sets), it is printed as JSON, pretty-printed unless `--compact` is specified.

## Examples

Assume a file `data.json`:

```json
{
  "users": [
    { "name": "Alice", "age": 30 },
    { "name": "Bob",   "age": 25 }
  ]
}
```

1. **Select all names:**

```bash
cat data.json | njq 'map (u: u.name) input.users'
```

```json
["Alice","Bob"]
```

2. **Filter by age:**

```bash
njq 'filter (u: u.age > 27) input.users' ./data.json
```

```json
[{ "name": "Alice", "age": 30 }]
```


3. **Import nix files to apply expressions:**

Where "myfile" is a nix expression:
```nix
let
    # you can do other imports here if you want... 
in 
builtins.attrNames builtins.input
```

```bash
njq 'import ./myfile.nix' ./data.json
```


## Notes

- The Nix expression is evaluated with `builtins` available, so you can use builtin functions like `map`, `filter`, etc.
- Errors during parsing or evaluation are printed to stderr, and the program exits with a non-zero status.
- Warnings from the evaluation are also printed to stderr.
- Use `--help` to see the usage and options.
