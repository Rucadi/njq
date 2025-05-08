# njq

`njq` (Nix JQ) lets you use Nix as a lightweight query language for JSON data. 
It parses JSON into a Nix "builtin" named `input`, evaluates your Nix expression, and prints the result of it.

It uses [Snix](https://snix.dev/) as nix implementation, altough slightly modified fork to support windows.

All the heavy-weight is done by Snix, njq interfaces Snix to provide this functionality.

You can check the available builtins for nix here: https://nix.dev/manual/nix/2.24/language/builtins.html





---

## Features

* **Arbitrary Nix expressions** over JSON data
* `--pretty` print code in a readable way
* `--nix` mode to evaluate a self‑contained Nix expression (ignore JSON input)
* Read JSON from a file or from standard input
* builtins.* already available without requiring to write builtin for each function
* builtins.input contains the "input" json already parsed as a nix expression.

---

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

---

## Usage

```
Usage: njq [--escaped] [--nix] <nix_expr> [json_file]

  --escaped        Print output with JSON escapes
  --nix        Treat <nix_expr> as a self‑contained expression (skip JSON input)
  <nix_expr>   The Nix expression to evaluate (quote it!)
  [json_file]  Path to JSON input file; if omitted, reads from stdin
  help         Show this help message
```

* **`<nix_expr>`** is evaluated with:

  ```nix
  with builtins;
  <nix_expr>
  ```
* The JSON input is made available as the Nix variable `input`.
* By default, JSON input is read from `json_file` or from `stdin`.
* In `--nix` mode, no JSON is read and `input` is `null`.

---

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
   cat data.json \
     | njq 'map (u: u.name) input.users'
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


3. **Pure Nix expression (no JSON):**

   ```bash
   njq --nix 'length [1 2 3 4]'
   ```

   ```json
   4
   ```
4. **Import nix files to apply expressions:**

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
