use snix_eval::builtin_macros::builtins;

#[builtins]
mod custom_builtins {
    use snix_eval::generators::{Gen, GenCo};
    use snix_eval::{ErrorKind, NixString, Value};
    use bstr::ByteSlice;

    
    #[builtin("prependHello")]
    pub async fn builtin_prepend_hello(co: GenCo, x: Value) -> Result<Value, ErrorKind> {
        match x {
            Value::String(s) => {
                let new_string = NixString::from(format!("hello {}", s.to_str().unwrap()));
                Ok(Value::from(new_string))
            }
            _ => Err(ErrorKind::TypeError {
                expected: "string",
                actual: "not string",
            }),
        }
    }
}