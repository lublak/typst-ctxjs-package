use std::collections::HashMap;

use rquickjs::FromIteratorJs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum JSBytesValue {
    Uninitialized,
    Undefined,
    Null,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),
    Array(Vec<JSBytesValue>),
    Object(HashMap<String, JSBytesValue>),
}

impl<'js> rquickjs::FromJs<'js> for JSBytesValue {
    fn from_js(ctx: &rquickjs::Ctx<'js>, v: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        match v.type_of() {
            rquickjs::Type::Uninitialized => Ok(JSBytesValue::Uninitialized),
            rquickjs::Type::Undefined => Ok(JSBytesValue::Undefined),
            rquickjs::Type::Null => Ok(JSBytesValue::Null),
            rquickjs::Type::Bool => v.as_bool().map(JSBytesValue::Bool).ok_or_else(|| {
                rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Bool.as_str())
            }),
            rquickjs::Type::Int => v.as_int().map(JSBytesValue::Int).ok_or_else(|| {
                rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Int.as_str())
            }),
            rquickjs::Type::Float => v.as_float().map(JSBytesValue::Float).ok_or_else(|| {
                rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Float.as_str())
            }),
            rquickjs::Type::String => v
                .as_string()
                .ok_or_else(|| {
                    rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::String.as_str())
                })?
                .to_string()
                .map(JSBytesValue::String),
            rquickjs::Type::Array => v
                .as_array()
                .ok_or_else(|| {
                    rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Array.as_str())
                })?
                .iter()
                .map(|v| JSBytesValue::from_js(ctx, v?))
                .collect::<Result<Vec<JSBytesValue>, rquickjs::Error>>()
                .map(JSBytesValue::Array),
            rquickjs::Type::Object => {
                let mut value = HashMap::<String, JSBytesValue>::new();

                let object = v.as_object().ok_or_else(|| {
                    rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Object.as_str())
                })?;
                let keys = object
                    .keys::<String>()
                    .map(|key| key)
                    .collect::<Result<Vec<String>, rquickjs::Error>>()?;

                for ele in keys {
                    value.insert(
                        ele.clone(),
                        JSBytesValue::from_js(
                            ctx,
                            object.get::<String, rquickjs::Value<'js>>(ele)?,
                        )?,
                    );
                }

                Ok(JSBytesValue::Object(value))
            }
            rquickjs::Type::Promise => v
                .as_promise()
                .ok_or_else(|| {
                    rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Promise.as_str())
                })?
                .finish(),
            t => Err(rquickjs::Error::new_from_js(t.as_str(), "JSBytesValue")),
        }
    }
}

impl<'js> rquickjs::IntoJs<'js> for JSBytesValue {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> Result<rquickjs::Value<'js>, rquickjs::Error> {
        match self {
            JSBytesValue::Uninitialized => Ok(rquickjs::Value::new_uninitialized(ctx.clone())),
            JSBytesValue::Undefined => Ok(rquickjs::Value::new_undefined(ctx.clone())),
            JSBytesValue::Null => Ok(rquickjs::Value::new_null(ctx.clone())),
            JSBytesValue::Bool(value) => Ok(rquickjs::Value::new_bool(ctx.clone(), value)),
            JSBytesValue::Int(value) => Ok(rquickjs::Value::new_int(ctx.clone(), value)),
            JSBytesValue::Float(value) => Ok(rquickjs::Value::new_float(ctx.clone(), value)),
            JSBytesValue::String(value) => {
                rquickjs::String::from_str(ctx.clone(), &value).map(rquickjs::Value::from_string)
            }
            JSBytesValue::Array(value) => {
                rquickjs::Array::from_iter_js(ctx, value.into_iter().map(|v| v.into_js(ctx)))
                    .map(rquickjs::Value::from_array)
            }
            JSBytesValue::Object(value) => rquickjs::Object::from_iter_js(
                ctx,
                value.iter().map(|(k, v)| (k, v.to_owned().into_js(ctx))),
            )
            .map(rquickjs::Value::from_object),
        }
    }
}

impl JSBytesValue {
    pub fn to_value_string(self) -> Result<String, String> {
        match self {
            JSBytesValue::Uninitialized => Ok("null".to_string()),
            JSBytesValue::Undefined => Ok("null".to_string()),
            JSBytesValue::Null => Ok("null".to_string()),
            JSBytesValue::Bool(value) => Ok(value.to_string()),
            JSBytesValue::Int(value) => Ok(value.to_string()),
            JSBytesValue::Float(value) => Ok(value.to_string()),
            JSBytesValue::String(value) => Ok(format!("\"{}\"", value.replace("\"", "\\\""))),
            JSBytesValue::Array(value) => Ok(format!(
                "[{}]",
                value
                    .into_iter()
                    .map(|v| v.to_value_string())
                    .collect::<Result<Vec<String>, _>>()?
                    .join(", ")
            )),
            JSBytesValue::Object(value) => Ok(format!(
                "{{{}}}",
                value
                    .into_iter()
                    .map::<Result<String, String>, _>(|(k, v)| {
                        Ok(format!("{}:{}", k, v.to_value_string()?))
                    })
                    .collect::<Result<Vec<String>, _>>()?
                    .join(", ")
            )),
        }
    }
    pub fn eval(self, eval_prefix: &String) -> Self {
        if let JSBytesValue::String(value) = &self {
            if !eval_prefix.is_empty() && value.starts_with(eval_prefix) {
                panic!("TODO")
            }
        }
        self
    }
}
