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
    Float(f64),
    Bool(bool),
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
            Value::Float(f_) => write!(f, "{}", f_),
            Value::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
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
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
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

    // Subtracts two Values.
    pub fn sub(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        match (a.clone(), b.clone()) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),

            _ => Err(error::Error::TypeError(format!(
                "Invalid operand types for `sub`: {}, {}",
                Value::type_of(a),
                Value::type_of(b)
            ))),
        }
    }

    // Multiplies two Values.
    pub fn mul(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        match (a.clone(), b.clone()) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),

            _ => Err(error::Error::TypeError(format!(
                "Invalid operand types for `mul`: {}, {}",
                Value::type_of(a),
                Value::type_of(b)
            ))),
        }
    }

    // Divides two Values.
    pub fn div(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        match (a.clone(), b.clone()) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Float((x as f64) / (y as f64))),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x / y)),

            _ => Err(error::Error::TypeError(format!(
                "Invalid operand types for `div`: {}, {}",
                Value::type_of(a),
                Value::type_of(b)
            ))),
        }
    }

    // Compares two Values for equality.
    pub fn eq(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        let result = match (a, b) {
            (Value::None, Value::None) => true,
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (
                Value::Function {
                    name: name_a,
                    code: code_a,
                    arity: arity_a,
                },
                Value::Function {
                    name: name_b,
                    code: code_b,
                    arity: arity_b,
                },
            ) => std::ptr::eq(name_a, name_b) && std::ptr::eq(code_a, code_b) && arity_a == arity_b,
            (left, right) => {
                return Err(error::Error::TypeError(format!(
                    "Invalid operand types for `eq`: {}, {}",
                    Value::type_of(left),
                    Value::type_of(right)
                )));
            }
        };

        Ok(Value::Bool(result))
    }

    // Compares two Values for greater than.
    pub fn gt(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        let result = match (a, b) {
            (Value::Int(x), Value::Int(y)) => x > y,
            (Value::Float(x), Value::Float(y)) => x > y,
            (left, right) => {
                return Err(error::Error::TypeError(format!(
                    "Invalid operand types for `eq`: {}, {}",
                    Value::type_of(left),
                    Value::type_of(right)
                )));
            }
        };

        Ok(Value::Bool(result))
    }

    // Compares two Values for less than.
    pub fn lt(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        let result = match (a, b) {
            (Value::Int(x), Value::Int(y)) => x < y,
            (Value::Float(x), Value::Float(y)) => x < y,
            (left, right) => {
                return Err(error::Error::TypeError(format!(
                    "Invalid operand types for `eq`: {}, {}",
                    Value::type_of(left),
                    Value::type_of(right)
                )));
            }
        };

        Ok(Value::Bool(result))
    }

    // Compares two Values for greater than or equal to.
    pub fn ge(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        Ok(Value::or(
            Value::gt(a.clone(), b.clone())?,
            Value::eq(a, b)?,
        )?)
    }

    // Compares two Values for less than or equal to.
    pub fn le(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        Ok(Value::or(
            Value::lt(a.clone(), b.clone())?,
            Value::eq(a, b)?,
        )?)
    }

    pub fn not(a: Value<'a>) -> Result<Value<'a>, error::Error> {
        let result = match a {
            Value::Bool(v) => !v,
            other => {
                return Err(error::Error::TypeError(format!(
                    "Invalid operand type for `not`: {}",
                    Value::type_of(other)
                )));
            }
        };

        Ok(Value::Bool(result))
    }

    pub fn or(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        let result = match (a, b) {
            (Value::Bool(x), Value::Bool(y)) => x | y,
            (left, right) => {
                return Err(error::Error::TypeError(format!(
                    "Invalid operand types for `or`: {}, {}",
                    Value::type_of(left),
                    Value::type_of(right)
                )));
            }
        };

        Ok(Value::Bool(result))
    }

    pub fn and(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        let result = match (a, b) {
            (Value::Bool(x), Value::Bool(y)) => x & y,
            (left, right) => {
                return Err(error::Error::TypeError(format!(
                    "Invalid operand types for `or`: {}, {}",
                    Value::type_of(left),
                    Value::type_of(right)
                )));
            }
        };

        Ok(Value::Bool(result))
    }

    pub fn xor(a: Value<'a>, b: Value<'a>) -> Result<Value<'a>, error::Error> {
        let result = match (a, b) {
            (Value::Bool(x), Value::Bool(y)) => x ^ y,
            (left, right) => {
                return Err(error::Error::TypeError(format!(
                    "Invalid operand types for `or`: {}, {}",
                    Value::type_of(left),
                    Value::type_of(right)
                )));
            }
        };

        Ok(Value::Bool(result))
    }

    // Returns the type name of this Value.
    pub fn type_of(v: Value) -> String {
        match v {
            Value::None => String::from("NoneType"),
            Value::Int(_) => String::from("int"),
            Value::Float(_) => String::from("float"),
            Value::Bool(_) => String::from("bool"),
            Value::String(_) => String::from("str"),
            Value::Function { .. } => String::from("Callable"),
        }
    }
}
