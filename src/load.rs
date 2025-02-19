use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::value;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "args")]
pub(crate) enum Method {
    Eval(String),
    EvalFormat(String, HashMap<String, value::JSBytesValue>, String),
    DefineVars(HashMap<String, value::JSBytesValue>, String),
    CallFunction(String, Vec<value::JSBytesValue>, String),
    LoadModuleBytecode(Vec<u8>),
    LoadModuleJs(String, String),
    CallModuleFunction(String, String, Vec<value::JSBytesValue>, String),
}
