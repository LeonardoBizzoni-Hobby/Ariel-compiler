use std::collections::HashMap;

use super::value::Value;

pub type Environment<'a> = HashMap<&'a str, Value<'a>>;
