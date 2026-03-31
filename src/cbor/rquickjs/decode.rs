use minicbor::{
    data::{Int, Type},
    Decoder,
};
use rquickjs::{context::EvalOptions, CatchResultExt, CaughtError, Ctx, Value};

use crate::cbor::con;

pub fn decode_to_rquickjs<'b, 'js>(
    b: &'b [u8],
    ctx: &Ctx<'js>,
) -> Result<Value<'js>, minicbor::decode::Error> {
    let mut decoder = Decoder::new(b);
    decode(&mut decoder, ctx)
}

fn eval<'js>(
    ctx: &Ctx<'js>,
    js: Vec<u8>,
) -> std::result::Result<rquickjs::Value<'js>, CaughtError<'js>> {
    let mut options = EvalOptions::default();
    options.global = true;
    ctx.eval_with_options::<rquickjs::Value, _>(js, options)
        .catch(&ctx)
}

fn json<'js>(
    ctx: &Ctx<'js>,
    json: Vec<u8>,
) -> std::result::Result<rquickjs::Value<'js>, CaughtError<'js>> {
    ctx.json_parse(json).catch(&ctx)
}

pub(crate) fn decode<'a, 'js>(
    decoder: &'a mut Decoder,
    ctx: &Ctx<'js>,
) -> Result<Value<'js>, minicbor::decode::Error> {
    return Ok(match decoder.datatype()? {
        Type::Bool => rquickjs::Value::new_bool(ctx.clone(), decoder.bool()?),
        Type::Null => Value::new_null(ctx.clone()),
        Type::Undefined => Value::new_undefined(ctx.clone()),
        Type::U8 => Value::new_int(ctx.clone(), decoder.u8()?.into()),
        Type::U16 => Value::new_int(ctx.clone(), decoder.u16()?.into()),
        Type::U32 => match decoder.u32()? {
            v if (v as i32) as u32 == v => Value::new_int(ctx.clone(), v as i32),
            v => Value::new_big_int(ctx.clone(), v.into()),
        },
        Type::U64 => Value::new_big_int(
            ctx.clone(),
            decoder
                .u64()?
                .try_into()
                .map_err(|err: std::num::TryFromIntError| {
                    minicbor::decode::Error::type_mismatch(Type::U64).with_message(err)
                })?,
        ),
        Type::I8 => Value::new_int(ctx.clone(), decoder.i8()?.into()),
        Type::I16 => Value::new_int(ctx.clone(), decoder.i16()?.into()),
        Type::I32 => Value::new_int(ctx.clone(), decoder.i32()?.into()),
        Type::I64 => Value::new_big_int(ctx.clone(), decoder.i64()?.into()),
        Type::Int => match decoder.int()? {
            v if v > Int::from(i32::MAX) => Value::new_big_int(
                ctx.clone(),
                v.try_into()
                    .map_err(|err: minicbor::data::TryFromIntError| {
                        minicbor::decode::Error::type_mismatch(Type::Int).with_message(err)
                    })?,
            ),
            v => Value::new_int(
                ctx.clone(),
                v.try_into()
                    .map_err(|err: minicbor::data::TryFromIntError| {
                        minicbor::decode::Error::type_mismatch(Type::Int).with_message(err)
                    })?,
            ),
        },
        #[cfg(feature = "half")]
        Type::F16 => Value::new_float(ctx.clone(), decoder.f16()?.into()),
        Type::F32 => Value::new_float(ctx.clone(), decoder.f32()?.into()),
        Type::F64 => Value::new_float(ctx.clone(), decoder.f64()?.into()),
        Type::Simple => Value::new_int(ctx.clone(), decoder.simple()?.into()),
        Type::Bytes => match decoder.bytes()? {
            [b'$', b'_', b'{', s, b'}', b'_', b @ .., b'_', b'$', b'_', b'{', b'!', b'}'] => {
                match s {
                    &con::EVAL => eval(ctx, b.to_vec()).map_err(|err| {
                        minicbor::decode::Error::type_mismatch(Type::Bytes).with_message(err)
                    })?,
                    &con::JSON => json(ctx, b.to_vec()).map_err(|err| {
                        minicbor::decode::Error::type_mismatch(Type::Bytes).with_message(err)
                    })?,
                    _ => rquickjs::TypedArray::new(ctx.clone(), b)
                        .map_err(|err| {
                            minicbor::decode::Error::type_mismatch(Type::Bytes).with_message(err)
                        })?
                        .into_value(),
                }
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
        other => {
            return Err(minicbor::decode::Error::type_mismatch(other).with_message("unknown type"))
        }
    });
}
