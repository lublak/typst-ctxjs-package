use std::collections::HashMap;

use minicbor::Decoder;
use rquickjs::{Ctx, Value};

use crate::cbor;

pub(crate) fn array<'js>(
    ctx: &Ctx<'js>,
    decoder: &mut Decoder,
) -> Result<Vec<Value<'js>>, minicbor::decode::Error> {
    let arr = decoder.datatype()?;
    let mut array = vec![];
    crate::cbor::utils::loop_array!(arr, decoder, {
        array.push(cbor::rquickjs::decode(decoder, ctx)?);
    })?;

    Ok(array)
}
pub(crate) fn string_map<'js>(
    decoder: &mut Decoder,
) -> Result<HashMap<String, String>, minicbor::decode::Error> {
    let arr = decoder.datatype()?;
    let mut map = HashMap::new();
    crate::cbor::utils::loop_map!(arr, decoder, {
        let dt = decoder.datatype()?;
        map.insert(
            cbor::utils::string!(dt, decoder)?.to_string(),
            cbor::jsstring::decode(decoder)?,
        );
    })?;

    Ok(map)
}
