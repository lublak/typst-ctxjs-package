use std::fmt;

use minicbor::{
    data::{Int, Type},
    Decoder,
};
use rquickjs::{context::EvalOptions, CatchResultExt, Ctx, Value};

use crate::{cbor::con, strfmt};

pub type Result<'js, T> = std::result::Result<T, Error<'js>>;

pub enum Error<'js> {
    CborDecode(minicbor::decode::Error),
    RquickJSError(rquickjs::Error),
    RquickJSCaughtError(rquickjs::CaughtError<'js>),
}

impl<'js> From<minicbor::decode::Error> for Error<'js> {
    fn from(err: minicbor::decode::Error) -> Error<'js> {
        Error::CborDecode(err)
    }
}

impl<'js> From<rquickjs::Error> for Error<'js> {
    fn from(err: rquickjs::Error) -> Error<'js> {
        Error::RquickJSError(err)
    }
}

impl<'js> From<rquickjs::CaughtError<'js>> for Error<'js> {
    fn from(err: rquickjs::CaughtError<'js>) -> Error<'js> {
        Error::RquickJSCaughtError(err)
    }
}

impl<'js> fmt::Display for Error<'js> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CborDecode(err) => {
                write!(f, "cbor error: {}", err)
            }
            Error::RquickJSError(err) => {
                write!(f, "rquickjs error: {}", err)
            }
            Error::RquickJSCaughtError(err) => write!(f, "rquickjs caught error: {}", err),
        }
    }
}

impl<'js> fmt::Debug for Error<'js> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CborDecode(err) => {
                write!(f, "cbor error: {:?}", err)
            }
            Error::RquickJSError(err) => {
                write!(f, "rquickjs error: {:?}", err)
            }
            Error::RquickJSCaughtError(err) => {
                write!(f, "rquickjs caught error: {:?}", err)
            }
        }
    }
}

// pub fn decode_to_rquickjs<'b, 'js>(
//     b: &'b [u8],
//     ctx: &Ctx<'js>,
// ) -> Result<Value<'js>, minicbor::decode::Error> {
//     let mut decoder = Decoder::new(b);
//     decode(&mut decoder, ctx)
// }

fn eval<'a, 'js>(decoder: &'a mut Decoder, ctx: &Ctx<'js>) -> Result<'js, Value<'js>> {
    let mut options = EvalOptions::default();
    options.global = true;
    Ok(ctx
        .eval_with_options::<rquickjs::Value, _>(decoder.str()?, options)
        .catch(&ctx)?)
}

fn eval_format<'a, 'js>(decoder: &'a mut Decoder, ctx: &Ctx<'js>) -> Result<'js, Value<'js>> {
    crate::cbor::utils::array_fixed_length(decoder, 2)?;

    let js = decoder.bytes()?;
    let arguments = super::args::string_map(decoder)?;

    let mut options = EvalOptions::default();
    options.global = true;
    Ok(ctx
        .eval_with_options::<rquickjs::Value, _>(
            strfmt::strfmt(js, &arguments).map_err(|err| {
                minicbor::decode::Error::type_mismatch(Type::Bytes).with_message(err)
            })?,
            options,
        )
        .catch(&ctx)?)
}

fn json<'a, 'js>(decoder: &'a mut Decoder, ctx: &Ctx<'js>) -> Result<'js, Value<'js>> {
    Ok(ctx.json_parse(decoder.str()?).catch(&ctx)?)
}

pub(crate) fn decode<'a, 'js>(decoder: &'a mut Decoder, ctx: &Ctx<'js>) -> Result<'js, Value<'js>> {
    Ok(match decoder.datatype()? {
        Type::Bool => rquickjs::Value::new_bool(ctx.clone(), decoder.bool()?),
        Type::Null => Value::new_null(ctx.clone()),
        Type::Undefined => Value::new_undefined(ctx.clone()),
        Type::U8 => Value::new_int(ctx.clone(), decoder.u8()?.into()),
        Type::U16 => Value::new_int(ctx.clone(), decoder.u16()?.into()),
        Type::U32 => match decoder.u32()? {
            v if (v as i32) as u32 == v => Value::new_int(ctx.clone(), v as i32),
            v => Value::new_big_int(ctx.clone(), v.into())?,
        },
        Type::U64 => Value::new_big_int(
            ctx.clone(),
            decoder
                .u64()?
                .try_into()
                .map_err(|err: std::num::TryFromIntError| {
                    minicbor::decode::Error::type_mismatch(Type::U64).with_message(err)
                })?,
        )?,
        Type::I8 => Value::new_int(ctx.clone(), decoder.i8()?.into()),
        Type::I16 => Value::new_int(ctx.clone(), decoder.i16()?.into()),
        Type::I32 => Value::new_int(ctx.clone(), decoder.i32()?.into()),
        Type::I64 => Value::new_big_int(ctx.clone(), decoder.i64()?.into())?,
        Type::Int => match decoder.int()? {
            v if v > Int::from(i32::MAX) => Value::new_big_int(
                ctx.clone(),
                v.try_into()
                    .map_err(|err: minicbor::data::TryFromIntError| {
                        minicbor::decode::Error::type_mismatch(Type::Int).with_message(err)
                    })?,
            )?,
            v => Value::new_int(
                ctx.clone(),
                v.try_into()
                    .map_err(|err: minicbor::data::TryFromIntError| {
                        minicbor::decode::Error::type_mismatch(Type::Int).with_message(err)
                    })?,
            ),
        },
        Type::F16 => Value::new_float(ctx.clone(), decoder.f16()?.into()),
        Type::F32 => Value::new_float(ctx.clone(), decoder.f32()?.into()),
        Type::F64 => Value::new_float(ctx.clone(), decoder.f64()?.into()),
        Type::Simple => Value::new_int(ctx.clone(), decoder.simple()?.into()),
        Type::Bytes => match decoder.bytes()? {
            // $ctxjs_cbor_
            [b'$', b'c', b't', b'x', b'j', b's', b'_', b'c', b'b', b'o', b'r', b'_', b @ ..] => {
                decode(&mut Decoder::new(b), ctx)?
            }
            b => rquickjs::TypedArray::new(ctx.clone(), b)
                .map_err(|err| {
                    minicbor::decode::Error::type_mismatch(Type::Bytes).with_message(err)
                })?
                .into_value(),
        },
        Type::String => rquickjs::String::from_str(ctx.clone(), decoder.str()?)
            .map_err(|err| minicbor::decode::Error::type_mismatch(Type::String).with_message(err))?
            .into_value(),
        Type::Array => {
            let array = rquickjs::Array::new(ctx.clone()).map_err(|err| {
                minicbor::decode::Error::type_mismatch(Type::Array).with_message(err)
            })?;
            for i in 0..crate::cbor::utils::array_length(decoder)? {
                array.set(i as _, decode(decoder, ctx)?).map_err(|err| {
                    minicbor::decode::Error::type_mismatch(Type::Array).with_message(err)
                })?;
            }

            rquickjs::Value::from_array(array)
        }
        Type::Map => {
            let object = rquickjs::Object::new(ctx.clone()).map_err(|err| {
                minicbor::decode::Error::type_mismatch(Type::Map).with_message(err)
            })?;
            for _ in 0..crate::cbor::utils::map_length(decoder)? {
                object
                    .set(decode(decoder, ctx)?, decode(decoder, ctx)?)
                    .map_err(|err| {
                        minicbor::decode::Error::type_mismatch(Type::Map).with_message(err)
                    })?;
            }
            rquickjs::Value::from_object(object)
        }
        Type::Tag => match decoder.tag()? {
            con::RAW_BYTES => rquickjs::TypedArray::new(ctx.clone(), decoder.bytes()?)
                .map_err(|err| {
                    minicbor::decode::Error::type_mismatch(Type::Bytes).with_message(err)
                })?
                .into_value(),
            con::EVAL => eval(decoder, ctx)?,
            con::EVAL_FORMAT => eval_format(decoder, ctx)?,
            con::JSON => json(decoder, ctx)?,
            t => {
                return Err(minicbor::decode::Error::tag_mismatch(t)
                    .with_message(format!("unsupported tagged data {}", t))
                    .into())
            }
        },
        other => {
            return Err(minicbor::decode::Error::type_mismatch(other)
                .with_message("unknown type")
                .into())
        }
    })
}
