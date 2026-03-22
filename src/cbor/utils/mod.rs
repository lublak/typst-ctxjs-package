pub mod typed_array;

#[macro_export]
macro_rules! loop_cbor_array {
    ($t:ident, $decoder:ident, $code:block) => {
        match $t {
            minicbor::data::Type::Array => {
                let len = $decoder.array()?.ok_or_else(|| {
                    minicbor::decode::Error::type_mismatch(minicbor::data::Type::Array)
                        .with_message("missing length")
                })?;

                for _ in (0..len) $code

                Ok(())
            }

            minicbor::data::Type::ArrayIndef => {
                if None == $decoder.array()? {
                    while minicbor::data::Type::Break != $decoder.datatype()? $code

                    $decoder.skip()
                } else {
                    Err(
                        minicbor::decode::Error::type_mismatch(minicbor::data::Type::ArrayIndef)
                            .with_message("unexpected length"),
                    )
                }
            }
            other => Err(minicbor::decode::Error::type_mismatch(other)
                .with_message("type is not an array")
                .at($decoder.position())),
        }
    }
}

#[macro_export]
macro_rules! loop_cbor_map {
    ($t:ident, $decoder:ident, $code:block) => {
        match $t {
            minicbor::data::Type::Map => {
                let len = $decoder.map()?.ok_or_else(|| {
                    minicbor::decode::Error::type_mismatch(minicbor::data::Type::Map)
                        .with_message("missing length")
                })?;

                for _ in (0..len) $code

                Ok(())
            }

            minicbor::data::Type::MapIndef => {
                if None == $decoder.map()? {
                    while minicbor::data::Type::Break != $decoder.datatype()? $code

                    $decoder.skip()
                } else {
                    Err(
                        minicbor::decode::Error::type_mismatch(minicbor::data::Type::MapIndef)
                            .with_message("unexpected length"),
                    )
                }
            }
            other => Err(minicbor::decode::Error::type_mismatch(other)
                .with_message("type is not a map")
                .at($decoder.position())),
        }
    };
}

#[macro_export]
macro_rules! loop_cbor_bytes_values {
    ($t:ident, $decoder:ident, $var:ident, $code:block) => {
        match $t {
            minicbor::data::Type::Bytes => {
                for $var in $decoder.bytes()?.iter() $code

                Ok(())
            }

            minicbor::data::Type::BytesIndef => {
                for b in $decoder.bytes_iter()? {
                    for $var in b?.iter() $code
                }
                Ok(())
            }
            other => Err(minicbor::decode::Error::type_mismatch(other)
                .with_message("type is not a bytes")
                .at($decoder.position())),
        }
    };
}

#[macro_export]
macro_rules! cbor_bytes {
    ($t:ident, $decoder:ident) => {
        match $t {
            minicbor::data::Type::Bytes => Ok(std::borrow::Cow::Borrowed($decoder.bytes()?)),

            minicbor::data::Type::BytesIndef => {
                let mut buf = Vec::new();
                for b in $decoder.bytes_iter()? {
                    buf.extend_from_slice(b?)
                }
                Ok(std::borrow::Cow::Owned(buf))
            }
            other => Err(minicbor::decode::Error::type_mismatch(other)
                .with_message("type is not a bytes")
                .at($decoder.position())),
        }
    };
}

#[macro_export]
macro_rules! cbor_string {
    ($t:ident, $decoder:ident) => {
        match $t {
            minicbor::data::Type::String => Ok(std::borrow::Cow::Borrowed($decoder.str()?)),

            minicbor::data::Type::StringIndef => {
                let mut strr = String::new();
                for b in $decoder.str_iter()? {
                    strr += b?
                }
                Ok(std::borrow::Cow::Owned(strr))
            }
            other => Err(minicbor::decode::Error::type_mismatch(other)
                .with_message("type is not a string")
                .at($decoder.position())),
        }
    };
}

pub use cbor_bytes;
pub use cbor_string;
pub use loop_cbor_array;
pub use loop_cbor_bytes_values;
pub use loop_cbor_map;
pub use typed_array::*;
