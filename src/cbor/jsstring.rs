use minicbor::{data::Type, Decoder};

use crate::cbor::{self, con};

pub(crate) fn decode<'a, 'js>(decoder: &'a mut Decoder) -> Result<String, minicbor::decode::Error> {
    return Ok(match decoder.datatype()? {
        Type::Bool => if decoder.bool()? { "true" } else { "false" }.to_string(),
        Type::Null => "null".to_string(),
        Type::Undefined => "undefined".to_string(),
        Type::U8 => decoder.u8()?.to_string(),
        Type::U16 => decoder.u16()?.to_string(),
        Type::U32 => decoder.u32()?.to_string(),
        Type::U64 => decoder.u64()?.to_string(),
        Type::I8 => decoder.i8()?.to_string(),
        Type::I16 => decoder.i16()?.to_string(),
        Type::I32 => decoder.i32()?.to_string(),
        Type::I64 => decoder.i64()?.to_string(),
        Type::Int => decoder.int()?.to_string(),
        #[cfg(feature = "half")]
        Type::F16 => decoder.f16()?.to_string(),
        Type::F32 => decoder.f32()?.to_string(),
        Type::F64 => decoder.f64()?.to_string(),
        Type::Simple => decoder.simple()?.to_string(),
        Type::Bytes => match decoder.bytes()? {
            [b'$', b'_', b'{', s, b'}', b'_', b @ .., b'_', b'$', b'_', b'{', b'!', b'}'] => {
                match s {
                    &con::EVAL => String::from_utf8(b.to_vec()).map_err(|e| {
                        minicbor::decode::Error::type_mismatch(Type::Bytes).with_message(e)
                    })?,
                    &con::JSON => {
                        if cbor::json::is_json(b) {
                            String::from_utf8(b.to_vec()).map_err(|e| {
                                minicbor::decode::Error::type_mismatch(Type::Bytes).with_message(e)
                            })?
                        } else {
                            Err(minicbor::decode::Error::type_mismatch(Type::Bytes)
                                .with_message(""))?
                        }
                    }
                    _ => {
                        let mut jsstring = String::new();
                        jsstring += "new Uint8Array([";
                        let mut first = true;

                        for ele in b {
                            if first {
                                first = false
                            } else {
                                jsstring += ","
                            }

                            jsstring += &ele.to_string();
                        }

                        jsstring + "])"
                    }
                }
            }
            b => {
                let mut jsstring = String::new();
                jsstring += "new Uint8Array([";
                let mut first = true;

                for ele in b {
                    if first {
                        first = false
                    } else {
                        jsstring += ","
                    }

                    jsstring += &ele.to_string();
                }

                jsstring + "])"
            }
        },
        Type::String => {
            format!("\"{}\"", decoder.str()?.replace("\"", "\\\""))
        }
        minicbor::data::Type::Array => {
            let mut jsstring = String::new();
            jsstring += "[";

            for i in 0..super::utils::array_length(decoder)? {
                if i != 0 {
                    jsstring += ","
                }

                jsstring += &decode(decoder)?;
            }

            jsstring + "]"
        }
        minicbor::data::Type::Map => {
            let mut jsstring = String::new();
            jsstring += "{";

            for i in 0..super::utils::map_length(decoder)? {
                if i == 0 {
                    jsstring += &format!("{}:{}", decode(decoder)?, decode(decoder)?,);
                } else {
                    jsstring += &format!(",{}:{}", decode(decoder,)?, decode(decoder,)?,);
                }
            }

            jsstring + "}"
        }
        other => {
            return Err(minicbor::decode::Error::type_mismatch(other)
                .with_message("unknown or unsupported type"))
        }
    });
}
