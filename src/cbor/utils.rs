use minicbor::Decoder;
use rquickjs::qjs;

pub enum TypedArrayType {
    UInt8C,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    BigInt64,
    BigUint64,
    //Float16,
    Float32,
    Float64,
}

pub fn get_typed_array_type<'js>(v: &rquickjs::Object<'js>) -> Option<TypedArrayType> {
    let array_type = unsafe { qjs::JS_GetTypedArrayType(v.as_raw()) };
    if array_type < 0 {
        return None;
    }
    match array_type as u32 {
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_UINT8C => Some(TypedArrayType::UInt8C),
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_INT8 => Some(TypedArrayType::Int8),
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_UINT8 => Some(TypedArrayType::UInt8),
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_INT16 => Some(TypedArrayType::Int16),
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_UINT16 => Some(TypedArrayType::UInt16),
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_INT32 => Some(TypedArrayType::Int32),
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_UINT32 => Some(TypedArrayType::UInt32),
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_BIG_INT64 => Some(TypedArrayType::BigInt64),
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_BIG_UINT64 => Some(TypedArrayType::BigUint64),
        //qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_FLOAT16 => Some(TypedArrayType::Float16),
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_FLOAT32 => Some(TypedArrayType::Float32),
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_FLOAT64 => Some(TypedArrayType::Float64),
        _ => None,
    }
}

pub fn array_length(decoder: &mut Decoder) -> Result<u64, minicbor::decode::Error> {
    decoder.array()?.ok_or_else(|| {
        minicbor::decode::Error::type_mismatch(minicbor::data::Type::Array)
            .with_message("missing length")
    })
}

pub fn array_fixed_length(decoder: &mut Decoder, len: u64) -> Result<u64, minicbor::decode::Error> {
    if array_length(decoder)? == len {
        return Ok(len);
    }
    Err(
        minicbor::decode::Error::type_mismatch(minicbor::data::Type::Array)
            .with_message("mismatch length"),
    )
}

pub fn map_length(decoder: &mut Decoder) -> Result<u64, minicbor::decode::Error> {
    decoder.map()?.ok_or_else(|| {
        minicbor::decode::Error::type_mismatch(minicbor::data::Type::Map)
            .with_message("missing length")
    })
}

// pub fn map_fixed_length(decoder: &mut Decoder, len: u64) -> Result<u64, minicbor::decode::Error> {
//     if map_length(decoder)? == len {
//         return Ok(len);
//     }
//     Err(
//         minicbor::decode::Error::type_mismatch(minicbor::data::Type::Map)
//             .with_message("mismatch length"),
//     )
// }
