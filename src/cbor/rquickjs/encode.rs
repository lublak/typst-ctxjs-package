use std::fmt;

use crate::cbor::utils::get_typed_array_type;

use crate::cbor::utils::TypedArrayType;
use minicbor::{encode::Write, Encoder};

type Result<T, W: Write> = std::result::Result<T, Error<W>>;

enum Error<W: Write> {
    CborEncode(minicbor::encode::Error<W::Error>),
    RquickJSError(rquickjs::Error),
}

impl<W: Write> From<minicbor::encode::Error<W::Error>> for Error<W> {
    fn from(err: minicbor::encode::Error<W::Error>) -> Error<W> {
        Error::CborEncode(err)
    }
}

impl<W: Write> From<rquickjs::Error> for Error<W> {
    fn from(err: rquickjs::Error) -> Error<W> {
        Error::RquickJSError(err)
    }
}

impl<W: Write> fmt::Display for Error<W>
where
    W: Write,
    W::Error: std::fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CborEncode(err) => {
                write!(f, "cbor error: {}", err)
            }
            Error::RquickJSError(err) => {
                write!(f, "rquickjs error: {}", err)
            }
        }
    }
}

impl<W: Write> fmt::Debug for Error<W>
where
    W: Write,
    W::Error: std::fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CborEncode(err) => {
                write!(f, "cbor error: {:?}", err)
            }
            Error::RquickJSError(err) => {
                write!(f, "rquickjs error: {:?}", err)
            }
        }
    }
}

macro_rules! encode_rquickjs_typed_array {
    ($encoder:ident, $t:ident, $object:ident) => {{
        let arr = $object
            .as_typed_array::<$t>()
            .ok_or_else(|| rquickjs::Error::new_from_js($object.type_name(), "cbor"))?;
        $encoder.array(arr.len() as _);
        for item in arr.as_ref() as &[$t] {
            $encoder.$t(*item);
        }
        Ok($encoder)
    }};
}

pub fn encode_to_bytes<'js>(v: &rquickjs::Value<'js>) -> Result<Vec<u8>, Vec<u8>> {
    let mut encoder = Encoder::new(Vec::new());
    encode(&mut encoder, v)?;
    Ok(encoder.into_writer())
}

fn encode_typed_array<'a, 'js, W: Write>(
    encoder: &'a mut Encoder<W>,
    object: &rquickjs::Object<'js>,
    t: TypedArrayType,
) -> Result<&'a mut Encoder<W>, W> {
    match t {
        TypedArrayType::UInt8C | TypedArrayType::UInt8 => {
            encoder.bytes(
                object
                    .as_typed_array::<u8>()
                    .ok_or_else(|| rquickjs::Error::new_from_js(object.type_name(), "cbor"))?
                    .as_bytes()
                    .ok_or_else(|| {
                        rquickjs::Error::new_from_js(
                            object.type_name(),
                            rquickjs::Type::Object.as_str(),
                        )
                    })?,
            );
            Ok(encoder)
        }
        TypedArrayType::Int8 => encode_rquickjs_typed_array!(encoder, i8, object),
        TypedArrayType::Int16 => encode_rquickjs_typed_array!(encoder, i16, object),
        TypedArrayType::UInt16 => encode_rquickjs_typed_array!(encoder, u16, object),
        TypedArrayType::Int32 => encode_rquickjs_typed_array!(encoder, i32, object),
        TypedArrayType::UInt32 => encode_rquickjs_typed_array!(encoder, u32, object),
        TypedArrayType::BigInt64 => encode_rquickjs_typed_array!(encoder, i64, object),
        TypedArrayType::BigUint64 => encode_rquickjs_typed_array!(encoder, u64, object),
        TypedArrayType::Float32 => encode_rquickjs_typed_array!(encoder, f32, object),
        TypedArrayType::Float64 => encode_rquickjs_typed_array!(encoder, f64, object),
    }
}

fn encode<'a, 'js, W: Write>(
    encoder: &'a mut Encoder<W>,
    v: &rquickjs::Value<'js>,
) -> Result<&'a mut Encoder<W>, W> {
    Ok(match v.type_of() {
        rquickjs::Type::Uninitialized => encoder.undefined()?,
        rquickjs::Type::Undefined => encoder.undefined()?,
        rquickjs::Type::Null => encoder.null()?,
        rquickjs::Type::Bool => encoder.bool(v.as_bool().ok_or_else(|| {
            rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Bool.as_str())
        })?)?,
        rquickjs::Type::Int => encoder.i32(v.as_int().ok_or_else(|| {
            rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Int.as_str())
        })?)?,
        rquickjs::Type::Float => encoder.f64(v.as_float().ok_or_else(|| {
            rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Float.as_str())
        })?)?,
        rquickjs::Type::String => encoder.str(
            &v.as_string()
                .ok_or_else(|| {
                    rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::String.as_str())
                })?
                .to_string()?,
        )?,
        rquickjs::Type::Array => {
            let arr = v.as_array().ok_or_else(|| {
                rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Array.as_str())
            })?;

            if let Some(t) = get_typed_array_type(arr) {
                encode_typed_array(encoder, arr, t)?
            } else {
                encoder.array(arr.len() as _)?;
                for item in arr.values() {
                    encode(encoder, &item?)?;
                }
                encoder
            }
        }
        rquickjs::Type::Promise => encode(
            encoder,
            &v.as_promise()
                .ok_or_else(|| {
                    rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Promise.as_str())
                })?
                .finish()?,
        )?,
        rquickjs::Type::Object => {
            let object = v.as_object().ok_or_else(|| {
                rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::Object.as_str())
            })?;

            if let Some(t) = get_typed_array_type(object) {
                encode_typed_array(encoder, object, t)?
            } else {
                encoder.map(object.len() as _)?;
                for key in object.keys::<String>() {
                    let key = key?;
                    let value = object.get(&key)?;
                    encoder.str(&key)?;
                    encode(encoder, &value)?;
                }
                encoder
            }
        }
        rquickjs::Type::BigInt => encoder.i64(
            v.as_big_int()
                .ok_or_else(|| {
                    rquickjs::Error::new_from_js(v.type_name(), rquickjs::Type::BigInt.as_str())
                })?
                .clone()
                .to_i64()?,
        )?,
        t => Err(rquickjs::Error::new_from_js(t.as_str(), "Vec<u8>"))?,
    })
}
