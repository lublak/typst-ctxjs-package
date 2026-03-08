use minicbor::Decoder;

pub fn cbor_bytes_to_jsstring<'b, 'js>(
    b: &'b [u8],
    type_field: &str,
) -> Result<String, minicbor::decode::Error> {
    let mut decoder = Decoder::new(b);
    cbor_decode_jsstring(&mut decoder, type_field)
}

pub(crate) fn cbor_decode_jsstring<'a, 'js>(
    decoder: &'a mut Decoder,
    type_field: &str,
) -> Result<String, minicbor::decode::Error> {
    return Ok(match decoder.datatype()? {
        minicbor::data::Type::Bool => if decoder.bool()? { "true" } else { "false" }.to_string(),
        minicbor::data::Type::Null => "null".to_string(),
        minicbor::data::Type::Undefined => "undefined".to_string(),
        minicbor::data::Type::U8 => decoder.u8()?.to_string(),
        minicbor::data::Type::U16 => decoder.u16()?.to_string(),
        minicbor::data::Type::U32 => decoder.u32()?.to_string(),
        minicbor::data::Type::U64 => decoder.u64()?.to_string(),
        minicbor::data::Type::I8 => decoder.i8()?.to_string(),
        minicbor::data::Type::I16 => decoder.i16()?.to_string(),
        minicbor::data::Type::I32 => decoder.i32()?.to_string(),
        minicbor::data::Type::I64 => decoder.i64()?.to_string(),
        minicbor::data::Type::Int => decoder.int()?.to_string(),
        #[cfg(feature = "half")]
        minicbor::data::Type::F16 => decoder.f16()?.to_string(),
        minicbor::data::Type::F32 => decoder.f32()?.to_string(),
        minicbor::data::Type::F64 => decoder.f64()?.to_string(),
        minicbor::data::Type::Simple => decoder.simple()?.to_string(),
        minicbor::data::Type::Bytes => format!(
            "new Uint8Array([{}])",
            decoder
                .bytes()?
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",")
        ),
        minicbor::data::Type::BytesIndef => {
            let mut jsstring = String::new();
            jsstring += "new Uint8Array([";

            let mut first = true;

            for b in decoder.bytes_iter()? {
                if first {
                    first = false
                } else {
                    jsstring += ","
                }
                jsstring += &b?
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
            }

            jsstring + "])"
        }
        minicbor::data::Type::String => format!("\"{}\"", decoder.str()?.replace("\"", "\\\"")),
        minicbor::data::Type::StringIndef => {
            let mut jsstring = String::new();
            for b in decoder.str_iter()? {
                jsstring += b?
            }
            jsstring
        }
        minicbor::data::Type::Array => {
            let mut jsstring = String::new();
            jsstring += "[";

            let mut first = true;

            for _ in 0..decoder
                .array()?
                .ok_or_else(|| {
                    minicbor::decode::Error::type_mismatch(minicbor::data::Type::Array)
                        .with_message("missing length")
                })?
                .try_into()
                .map_err(|e| {
                    minicbor::decode::Error::type_mismatch(minicbor::data::Type::Array)
                        .with_message(e)
                })?
            {
                if first {
                    first = false
                } else {
                    jsstring += ","
                }

                jsstring += &cbor_decode_jsstring(decoder, type_field)?;
            }

            jsstring + "]"
        }
        minicbor::data::Type::ArrayIndef => {
            if None == decoder.array()? {
                let mut jsstring = String::new();
                jsstring += "[";

                let mut first = true;

                while minicbor::data::Type::Break != decoder.datatype()? {
                    if first {
                        first = false
                    } else {
                        jsstring += ","
                    }

                    jsstring += &cbor_decode_jsstring(decoder, type_field)?;
                }

                decoder.skip()?;

                jsstring + "]"
            } else {
                return Err(minicbor::decode::Error::type_mismatch(
                    minicbor::data::Type::ArrayIndef,
                )
                .with_message("unexpected length"));
            }
        }
        minicbor::data::Type::Map => {
            let mut jsstring = String::new();
            jsstring += "{";

            let mut first = true;

            for _ in 0..decoder
                .map()?
                .ok_or_else(|| {
                    minicbor::decode::Error::type_mismatch(minicbor::data::Type::Map)
                        .with_message("missing length")
                })?
                .try_into()
                .map_err(|e| {
                    minicbor::decode::Error::type_mismatch(minicbor::data::Type::Map)
                        .with_message(e)
                })?
            {
                if first {
                    first = false;
                    jsstring += &format!(
                        "{}:{}",
                        cbor_decode_jsstring(decoder, type_field)?,
                        cbor_decode_jsstring(decoder, type_field)?,
                    );
                } else {
                    jsstring += &format!(
                        ",{}:{}",
                        cbor_decode_jsstring(decoder, type_field)?,
                        cbor_decode_jsstring(decoder, type_field)?,
                    );
                }
            }
            jsstring + "}"
        }
        minicbor::data::Type::MapIndef => {
            if None == decoder.array()? {
                let mut jsstring = String::new();
                jsstring += "{";

                let mut first = true;

                while minicbor::data::Type::Break != decoder.datatype()? {
                    if first {
                        first = false;
                        jsstring += &format!(
                            "{}:{}",
                            cbor_decode_jsstring(decoder, type_field)?,
                            cbor_decode_jsstring(decoder, type_field)?,
                        );
                    } else {
                        jsstring += &format!(
                            ",{}:{}",
                            cbor_decode_jsstring(decoder, type_field)?,
                            cbor_decode_jsstring(decoder, type_field)?,
                        );
                    }
                }

                decoder.skip()?;

                jsstring + "}"
            } else {
                return Err(minicbor::decode::Error::type_mismatch(
                    minicbor::data::Type::ArrayIndef,
                )
                .with_message("unexpected length"));
            }
        }
        other => {
            return Err(minicbor::decode::Error::type_mismatch(other).with_message("unknown type"))
        }
    });
}
