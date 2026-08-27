use crate::{error, rt::bytecode};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

#[derive(Clone)]
pub struct StringInterner {
    strings: HashMap<String, Arc<str>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
        }
    }

    pub fn intern(&mut self, value: &str) -> Arc<str> {
        self.strings
            .entry(value.to_owned())
            .or_insert_with_key(|key| Arc::from(key.as_str()))
            .clone()
    }
}

#[derive(Clone)]
pub enum Value<'a> {
    None,
    Int(i64),
    String(Arc<str>),
    Function {
        name: &'a String,
        code: &'a bytecode::Block<'a>,
        arity: u16,
    },
}

impl<'a> fmt::Display for Value<'a> {
    // Converts this Value to a String.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Int(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "{}", s),
            Value::Function { name, .. } => write!(f, "<function '{}'>", name),
        }
    }
}

impl<'a> Value<'a> {
    // Adds two Values.
    pub fn add(
        a: Value<'a>,
        b: Value<'a>,
        interner: &mut StringInterner,
    ) -> Result<Value<'a>, error::Error> {
        match (a.clone(), b.clone()) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
            (Value::String(x), Value::String(y)) => {
                let mut result = String::with_capacity(x.len() + y.len());
                result.push_str(&x);
                result.push_str(&y);
                Ok(Value::String(interner.intern(&result)))
            }

            _ => Err(error::Error::TypeError(format!(
                "Invalid operand types for `add`: {}, {}",
                Value::type_of(a),
                Value::type_of(b)
            ))),
        }
    }

    // Returns the type name of this Value.
    pub fn type_of(v: Value) -> String {
        match v {
            Value::None => String::from("NoneType"),
            Value::Int(_) => String::from("int"),
            Value::String(_) => String::from("str"),
            Value::Function { .. } => String::from("Callable"),
        }
    }
}
