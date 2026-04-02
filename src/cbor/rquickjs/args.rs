use std::collections::HashMap;

use minicbor::Decoder;
use rquickjs::{Ctx, Value};

use crate::cbor;

pub(crate) fn array<'js>(
    ctx: &Ctx<'js>,
    decoder: &mut Decoder,
) -> Result<Vec<Value<'js>>, minicbor::decode::Error> {
    let len = cbor::utils::array_length(decoder)?;
    let mut array = Vec::with_capacity(len as _);
    for _ in 0..len {
        array.push(cbor::rquickjs::decode(decoder, ctx)?);
    }
    Ok(array)
}
pub(crate) fn string_map<'js, 'd>(
    decoder: &'d mut Decoder,
) -> Result<HashMap<&'d str, String>, minicbor::decode::Error> {
    let len = cbor::utils::map_length(decoder)?;
    let mut map = HashMap::with_capacity(len as _);
    for _ in 0..len {
        map.insert(decoder.str()?, cbor::jsstring::decode(decoder)?);
    }
    Ok(map)
}
