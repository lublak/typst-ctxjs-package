use std::collections::HashMap;

use rquickjs::{context::EvalOptions, CatchResultExt};
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

impl JSBytesValue {
    pub fn to_value_string<'js>(self, ctx: &rquickjs::Ctx<'js>, type_field: &String) -> Result<String, String> {
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
                    .map(|v| v.to_value_string(ctx, type_field))
                    .collect::<Result<Vec<String>, _>>()?
                    .join(", ")
            )),
            JSBytesValue::Object(value) => {
                if let Some(type_field_value) = value.get(type_field) {
                    if let JSBytesValue::String(type_field_value) = type_field_value {
                        match type_field_value.as_ref() {
                            "eval" => {
                                if let Some(JSBytesValue::String(js)) = value.get("value") {
                                    return Ok(js.to_owned())
                                } else {
                                    return Err("eval typed values needs to be a string".to_string())
                                }
                            }
                            t => {
                                return Err(format!("invalid type:{}", t))
                            }
                        }
                    } else {
                        return Err(format!("{} is not a string value", type_field))
                    }
                }
                Ok(format!(
                "{{{}}}",
                value
                    .into_iter()
                    .map::<Result<String, String>, _>(|(k, v)| {
                        Ok(format!("{}:{}", k, v.to_value_string(ctx, type_field)?))
                    })
                    .collect::<Result<Vec<String>, _>>()?
                    .join(", ")
            ))
        },
        }
    }

    pub fn to_js<'js>(self, ctx: &rquickjs::Ctx<'js>, type_field: &String) -> Result<rquickjs::Value<'js>, String> {
        match self {
            JSBytesValue::Uninitialized => Ok(rquickjs::Value::new_uninitialized(ctx.clone())),
            JSBytesValue::Undefined => Ok(rquickjs::Value::new_undefined(ctx.clone())),
            JSBytesValue::Null => Ok(rquickjs::Value::new_null(ctx.clone())),
            JSBytesValue::Bool(value) => Ok(rquickjs::Value::new_bool(ctx.clone(), value)),
            JSBytesValue::Int(value) => Ok(rquickjs::Value::new_int(ctx.clone(), value)),
            JSBytesValue::Float(value) => Ok(rquickjs::Value::new_float(ctx.clone(), value)),
            JSBytesValue::String(value) => {
                rquickjs::String::from_str(ctx.clone(), &value).map(rquickjs::Value::from_string).map_err(|e| e.to_string())
            }
            JSBytesValue::Array(value) => {
                let array = rquickjs::Array::new(ctx.clone()).map_err(|e| e.to_string())?;
                for (idx, item) in value.into_iter().enumerate() {
                    let item = item.to_js(ctx, type_field)?;
                    array.set(idx as _, item).map_err(|e| e.to_string())?;
                }
                Ok(rquickjs::Value::from_array(array))
            }
            JSBytesValue::Object(value) => {
                if let Some(type_field_value) = value.get(type_field) {
                    if let JSBytesValue::String(type_field_value) = type_field_value {
                        match type_field_value.as_ref() {
                            "eval" => {
                                if let Some(JSBytesValue::String(js)) = value.get("value") {
                                    let mut options = EvalOptions::default();
                                    options.global = true;
                                    return ctx.eval_with_options::<rquickjs::Value, _>(js.to_owned(), options)
                                    .catch(&ctx)
                                    .map_err(|e| format!("eval error: {}", e.to_string()))
                                } else {
                                    return Err("eval typed values needs to be a string".to_string())
                                }
                            }
                            t => {
                                return Err(format!("invalid type:{}", t))
                            }
                        }
                    } else {
                        return Err(format!("{} is not a string value", type_field))
                    }
                }
                let object = rquickjs::Object::new(ctx.clone()).map_err(|e| e.to_string())?;
                for (key, value) in value {
                    let value = value.to_js(ctx, type_field)?;
                    object.set(key, value).map_err(|e| e.to_string())?;
                }
                Ok(rquickjs::Value::from_object(object))
        },
        }
    }
}
