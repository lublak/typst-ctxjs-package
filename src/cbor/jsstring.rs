use minicbor::{data::Type, Decoder};

pub fn decode_to_string<'b, 'js>(
    b: &'b [u8],
    type_field: &str,
) -> Result<String, minicbor::decode::Error> {
    let mut decoder = Decoder::new(b);
    decode(&mut decoder, type_field)
}

pub(crate) fn decode<'a, 'js>(
    decoder: &'a mut Decoder,
    type_field: &str,
) -> Result<String, minicbor::decode::Error> {
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
        bytes if bytes == Type::Bytes || bytes == Type::BytesIndef => {
            let mut jsstring = String::new();
            jsstring += "new Uint8Array([";

            let mut first = true;

            crate::cbor::utils::loop_cbor_bytes_values!(bytes, decoder, b, {
                if first {
                    first = false
                } else {
                    jsstring += ","
                }

                jsstring += &b.to_string();
            })?;

            jsstring + "])"
        }
        string if string == Type::String || string == Type::StringIndef => {
            format!(
                "\"{}\"",
                crate::cbor::utils::cbor_string!(string, decoder)?.replace("\"", "\\\"")
            )
        }
        arr if arr == minicbor::data::Type::Array || arr == minicbor::data::Type::ArrayIndef => {
            let mut jsstring = String::new();
            jsstring += "[";

            let mut first = true;

            super::utils::loop_cbor_array!(arr, decoder, {
                if first {
                    first = false
                } else {
                    jsstring += ","
                }

                jsstring += &decode(decoder, type_field)?;
            })?;

            jsstring + "]"
        }
        map if map == minicbor::data::Type::Map || map == minicbor::data::Type::MapIndef => {
            let mut jsstring = String::new();
            jsstring += "{";

            let mut first = true;

            super::utils::loop_cbor_map!(map, decoder, {
                if first {
                    first = false;
                    jsstring += &format!(
                        "{}:{}",
                        decode(decoder, type_field)?,
                        decode(decoder, type_field)?,
                    );
                } else {
                    jsstring += &format!(
                        ",{}:{}",
                        decode(decoder, type_field)?,
                        decode(decoder, type_field)?,
                    );
                }
            })?;

            jsstring + "}"
        }
        other => {
            return Err(minicbor::decode::Error::type_mismatch(other).with_message("unknown type"))
        }
    });
}
