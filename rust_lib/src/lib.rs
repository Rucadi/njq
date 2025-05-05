use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::env;
use tvix_eval::{Evaluation, Value};

/// Helper: evaluate a Nix expr and return its Value, or None on failure
fn evaluate_to_value(code: &str) -> Option<Value> {
    let cwd = env::current_dir()
        .unwrap_or_else(|_| "/".into())
        .to_string_lossy()
        .into_owned();
    let evaluator = Evaluation::builder_impure().build();
    let result = evaluator.evaluate(code, Some(cwd.into()));
    result.value
}

/// Evaluates two Nix expressions: first `input_expr` becomes a builtin named "input",
/// then `code_expr` is evaluated with that builtin in scope.
#[no_mangle]
pub extern "C" fn eval_nix_expr(
    input_ptr: *const c_char,
    code_ptr: *const c_char,
) -> *mut c_char {
    // Null-check
    if input_ptr.is_null() || code_ptr.is_null() {
        return std::ptr::null_mut();
    }

    // Convert C pointers to &str
    let input_cstr = unsafe { CStr::from_ptr(input_ptr) };
    let code_cstr  = unsafe { CStr::from_ptr(code_ptr) };
    let input_expr = match input_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let code_expr = match code_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    // 1) Evaluate the "input" expression to a Value
    let input_val = match evaluate_to_value(input_expr) {
        Some(val) => val,
        None => return std::ptr::null_mut(),
    };

    // 2) Build a new evaluator with that as a builtin named "input"
    let builder   = Evaluation::builder_impure()
        .add_builtins([("input", input_val)]);
    let evaluator = builder.build();

    // 3) Run the real code, which can now do: builtins.input.someAttr
    let cwd = env::current_dir()
        .unwrap_or_else(|_| "/".into())
        .to_string_lossy()
        .into_owned();
    let result = evaluator.evaluate(code_expr, Some(cwd.into()));

    // 4) Turn the resulting Value into a string and hand it back as a newly‑allocated C string
    let out_str = match result.value {
        Some(v) => v.to_string(),
        None    => return std::ptr::null_mut(),
    };
    match CString::new(out_str) {
        Ok(cstr) => cstr.into_raw(),
        Err(_)   => std::ptr::null_mut(),
    }
}

/// Caller must free with this
#[no_mangle]
pub extern "C" fn free_cstring(s: *mut c_char) {
    if s.is_null() { return; }
    unsafe { let _ = CString::from_raw(s); }
}
