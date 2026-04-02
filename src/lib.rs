use base64::Engine as _;
use minicbor::{Decoder, Encoder};
use rquickjs::{context::EvalOptions, function::Args, CatchResultExt, Context, Module, Runtime};
use wasm_minimal_protocol::*;

use crate::cbor_load::cbor_decode_run_load;

mod cbor;
mod cbor_load;
mod strfmt;

initiate_protocol!();

struct ContextHolder(Option<Context>);
unsafe impl Send for ContextHolder {}
unsafe impl Sync for ContextHolder {}

static mut CURRENT_CONTEXT: ContextHolder = ContextHolder(None);
static mut CURRENT_VALUE: Option<Vec<u8>> = None;

#[inline(always)]
#[allow(static_mut_refs)]
fn get_current_context() -> Result<Context, String> {
    return Ok(unsafe { CURRENT_CONTEXT.0.to_owned().ok_or_else(|| "context empty") }?);
}

#[inline(always)]
fn set_current_context(ctx: Context) {
    unsafe {
        CURRENT_CONTEXT.0 = Some(ctx);
    }
}

#[inline(always)]
#[allow(static_mut_refs)]
fn get_stored_value() -> Vec<u8> {
    return unsafe {
        if let Some(value) = &CURRENT_VALUE {
            value.clone()
        } else {
            vec![]
        }
    };
}

#[inline(always)]
fn set_stored_value(val: Vec<u8>) {
    unsafe {
        CURRENT_VALUE = Some(val);
    }
}

#[inline(always)]
fn set_stored_value_from_rquickjs(store: bool, val: &rquickjs::Value) -> Result<Vec<u8>, String> {
    let val = cbor::rquickjs::encode_to_bytes(val)
        .map_err(|e| format!("eval error: {}", e.to_string()))?;
    if store {
        set_stored_value(val.clone());
    }
    Ok(val)
}

#[wasm_func]
fn new_context(load: &[u8]) -> Result<Vec<u8>, String> {
    let runtime =
        Runtime::new().map_err(|e| format!("failed to create runtime: {}", e.to_string()))?;

    let ctx: Context = Context::full(&runtime)
        .map_err(|e| format!("failed to create context: {}", e.to_string()))?;

    cbor_decode_run_load(&mut Decoder::new(load), &ctx)
        .map_err(|e| format!("failed to run load: {}", e.to_string()))?;

    set_current_context(ctx);

    Ok(vec![])
}

#[wasm_func]
fn stored_value() -> Result<Vec<u8>, String> {
    Ok(get_stored_value())
}

#[wasm_func]
fn load(run: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    cbor_decode_run_load(&mut Decoder::new(run), &ctx)
        .map_err(|e| format!("failed to run load: {}", e.to_string()))?;

    Ok(vec![])
}

#[wasm_func]
fn eval(js: &[u8], store: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let js =
        std::str::from_utf8(js).map_err(|e| format!("failed to parse js: {}", e.to_string()))?;

    let store = !store.is_empty() && store[0] > 0;

    let mut options = EvalOptions::default();
    options.global = true;

    ctx.with(|ctx| {
        let value = &ctx
            .eval_with_options(js, options)
            .catch(&ctx)
            .map_err(|e| format!("eval error: {}", e.to_string()))?;
        set_stored_value_from_rquickjs(store, &value)
    })
}

#[wasm_func]
fn eval_format(js: &[u8], arguments: &[u8], store: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let mut decoder = Decoder::new(arguments);

    let arguments = cbor::rquickjs::args::string_map(&mut decoder)
        .map_err(|e| format!("failed to deserialize arguments: {}", e.to_string()))?;

    let store = !store.is_empty() && store[0] > 0;

    let mut options = EvalOptions::default();
    options.global = true;

    ctx.with(|ctx| {
        let value = ctx
            .eval_with_options(
                strfmt::strfmt(js, &arguments)
                    .map_err(|e| format!("can not format js string: {}", e))?,
                options,
            )
            .catch(&ctx)
            .map_err(|e| format!("eval error: {}", e.to_string()))?;
        set_stored_value_from_rquickjs(store, &value)
    })
}

#[wasm_func]
fn define_vars(variables: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let mut decoder = Decoder::new(variables);

    let variables = cbor::rquickjs::args::string_map(&mut decoder)
        .map_err(|e| format!("failed to deserialize variables: {}", e.to_string()))?;

    let variables: String = variables
        .into_iter()
        .map(|(k, v)| format!("let {}={}", k, v))
        .fold(String::new(), |a, b| a + &b + ";");

    ctx.with(|ctx| {
        _ = ctx
            .eval::<rquickjs::Value, std::string::String>(format!("{};", variables))
            .catch(&ctx)
            .map_err(|e| format!("eval error: {}", e.to_string()))?;

        Ok(vec![])
    })
}

#[wasm_func]
fn call_function(fn_name: &[u8], arguments: &[u8], store: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let fn_name: &str = std::str::from_utf8(fn_name)
        .map_err(|e| format!("failed to parse fn_name: {}", e.to_string()))?;

    let store = store.len() > 0 && store[0] > 0;

    ctx.with(|ctx| {
        let arguments: Vec<rquickjs::Value> =
            cbor::rquickjs::args::array(&ctx, &mut Decoder::new(arguments))
                .map_err(|e| format!("failed to deserialize arguments: {}", e.to_string()))?;

        let mut args = Args::new(ctx.clone(), arguments.len());
        args.push_args(arguments)
            .map_err(|e| format!("failed to add args: {}", e.to_string()))?;

        let func: rquickjs::Function = ctx
            .globals()
            .get(fn_name)
            .catch(&ctx)
            .map_err(|e| format!("failed to get function: {}", e.to_string()))?;

        let res = func
            .call_arg(args)
            .catch(&ctx)
            .map_err(|e| format!("failed to call function: {}", e.to_string()))?;

        set_stored_value_from_rquickjs(store, &res)
    })
}

//#[wasm_func]
//fn compile_module_bytecode(module_name: &[u8], module: &[u8]) -> Result<Vec<u8>, String> {
//    let ctx = get_current_context()?;
//
//    let module_name: &str = std::str::from_utf8(module_name)
//        .map_err(|e| format!("failed to parse module_name: {}", e.to_string()))?;
//
//    let module: &str = std::str::from_utf8(module)
//        .map_err(|e| format!("failed to parse module: {}", e.to_string()))?;
//
//    ctx.with(|ctx| {
//        let m = Module::declare(ctx, module_name, module)
//            .map_err(|e| format!("failed declare module: {}", e.to_string()))?;
//        let byte_code = m
//            .write(WriteOptions {
//                endianness: WriteOptionsEndianness::Native,
//                allow_shared_array_buffer: false,
//                object_reference: false,
//                strip_source: true,
//                strip_debug: true,
//            })
//            .map_err(|e| format!("failed to get bytecode: {}", e.to_string()))?;
//
//        Ok(byte_code)
//    })
//}

#[wasm_func]
fn load_module_bytecode(bytecode: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    ctx.with(|ctx| {
        let m = unsafe { Module::load(ctx.clone(), bytecode) }
            .catch(&ctx)
            .map_err(|e| format!("failed load bytecode: {}", e.to_string()))?;
        _ = m
            .eval()
            .catch(&ctx)
            .map_err(|e| format!("failed eval bytecode: {}", e.to_string()))?;

        Ok(vec![])
    })
}

#[wasm_func]
fn load_module_js(module_name: &[u8], module: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let module_name: &str = std::str::from_utf8(module_name)
        .map_err(|e| format!("failed to parse module_name: {}", e.to_string()))?;

    let module: &str = std::str::from_utf8(module)
        .map_err(|e| format!("failed to parse module: {}", e.to_string()))?;

    ctx.with(|ctx| {
        _ = Module::declare(ctx.clone(), module_name, module)
            .catch(&ctx)
            .map_err(|e| format!("failed load module code: {}", e.to_string()))?
            .eval()
            .catch(&ctx)
            .map_err(|e| format!("failed eval module code: {}", e.to_string()))?;
        Ok(vec![])
    })
}

#[wasm_func]
fn call_module_function(
    module_name: &[u8],
    fn_name: &[u8],
    arguments: &[u8],

    store: &[u8],
) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let module_name: &str = std::str::from_utf8(module_name)
        .map_err(|e| format!("failed to parse module_name: {}", e.to_string()))?;

    let fn_name: &str = std::str::from_utf8(fn_name)
        .map_err(|e| format!("failed to parse fn_name: {}", e.to_string()))?;

    let store = store.len() > 0 && store[0] > 0;

    ctx.with(|ctx| {
        let arguments: Vec<rquickjs::Value> =
            cbor::rquickjs::args::array(&ctx, &mut Decoder::new(arguments))
                .map_err(|e| format!("failed to deserialize arguments: {}", e.to_string()))?;

        let mut args = Args::new(ctx.clone(), arguments.len());
        args.push_args(arguments)
            .map_err(|e| format!("failed to add args: {}", e.to_string()))?;

        let m: rquickjs::Object = Module::import(&ctx, module_name)
            .catch(&ctx)
            .map_err(|e| format!("failed to import module: {}", e.to_string()))?
            .finish()
            .catch(&ctx)
            .map_err(|e| format!("failed to finish module import: {}", e.to_string()))?;

        let func: rquickjs::Function = m
            .get(fn_name)
            .catch(&ctx)
            .map_err(|e| format!("failed to get function: {}", e.to_string()))?;

        let res = func
            .call_arg(args)
            .catch(&ctx)
            .map_err(|e| format!("failed to call function: {}", e.to_string()))?;

        set_stored_value_from_rquickjs(store, &res)
    })
}

#[wasm_func]
fn get_module_properties(module_name: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let module_name: &str = std::str::from_utf8(module_name)
        .map_err(|e| format!("failed to parse module_name: {}", e.to_string()))?;

    ctx.with(|ctx| {
        let m: rquickjs::Object = Module::import(&ctx, module_name)
            .catch(&ctx)
            .map_err(|e| format!("failed to import module: {}", e.to_string()))?
            .finish()
            .catch(&ctx)
            .map_err(|e| format!("failed to finish module import: {}", e.to_string()))?;

        let mut encoder = Encoder::new(Vec::new());

        for key in m.keys() {
            let key: String = key
                .catch(&ctx)
                .map_err(|e| format!("can not collect module keys: {}", e.to_string()))?;

            encoder
                .str(&key)
                .map_err(|e| format!("failed to serialize results: {}", e.to_string()))?;
        }

        Ok(encoder.into_writer())
    })
}

#[wasm_func]
pub fn image_data_url(data: &[u8], format: &[u8]) -> Result<Vec<u8>, String> {
    let format: &str = std::str::from_utf8(format)
        .map_err(|e| format!("failed to parse format: {}", e.to_string()))?;

    let t: &str;
    if format.is_empty() {
        if infer::image::is_png(data) {
            t = "png"
        } else if infer::image::is_jpeg(data) {
            t = "jpeg"
        } else if infer::image::is_gif(data) {
            t = "gif"
        } else if infer::text::is_xml(data) {
            t = "svg"
        } else {
            return Err("data not supported".to_owned());
        }
    } else {
        match format.to_lowercase().as_str() {
            "png" => t = "png",
            "jpeg" => t = "jpeg",
            "gif" => t = "gif",
            "svg" => t = "svg",
            _ => return Err(format!("format {} not supported", format)),
        }
    }

    Ok(format!(
        "data:image/{};base64,{}",
        t,
        base64::prelude::BASE64_STANDARD.encode(&data)
    )
    .into_bytes())
}
