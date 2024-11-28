use std::{
    collections::HashMap, sync::{LazyLock, Mutex}
};

use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use rquickjs::{context::EvalOptions, function::Args, CatchResultExt, Context, Module, Runtime};
use value::JSBytesValue;
use wasm_minimal_protocol::*;

use strfmt::strfmt;

mod value;

initiate_protocol!();

struct ContextHolder(Context);
unsafe impl Send for ContextHolder {}
unsafe impl Sync for ContextHolder {}

static CONTEXT_STORE: LazyLock<Mutex<HashMap<String, ContextHolder>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[wasm_func]
fn create_context(ctx_name: &[u8]) -> Result<Vec<u8>, String> {
    let ctx_name = std::str::from_utf8(ctx_name)
        .map_err(|e| format!("failed to parse name: {}", e.to_string()))?;

    let runtime =
        Runtime::new().map_err(|e| format!("failed to create runtime: {}", e.to_string()))?;
    let ctx: Context = Context::full(&runtime)
        .map_err(|e| format!("failed to create context: {}", e.to_string()))?;

    let mut context_store = CONTEXT_STORE
        .lock()
        .map_err(|e| format!("failed to load context store: {}", e.to_string()))?;

    context_store.insert(ctx_name.to_string(), ContextHolder(ctx));

    Ok(vec![])
}

#[inline(always)]
fn get_context(ctx_name: &[u8]) -> Result<Context, String> {
    let ctx_name = std::str::from_utf8(ctx_name)
        .map_err(|e| format!("failed to parse name: {}", e.to_string()))?;
    let context_store = CONTEXT_STORE
        .lock()
        .map_err(|e| format!("failed to load context store: {}", e.to_string()))?;

    return Ok(context_store
        .get(ctx_name)
        .ok_or_else(|| format!("context {} not found", ctx_name))?
        .0
        .to_owned());
}

#[wasm_func]
fn eval(ctx_name: &[u8], js: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_context(ctx_name)?;

    let js =
        std::str::from_utf8(js).map_err(|e| format!("failed to parse js: {}", e.to_string()))?;

    let res: Result<JSBytesValue, String> = ctx.with(|ctx| {
        let mut options = EvalOptions::default();
        options.global = true;
        ctx.eval_with_options(js, options)
            .catch(&ctx)
            .map_err(|e| format!("eval error: {}", e.to_string()))
    });

    let mut buffer = vec![];
    _ = ciborium::ser::into_writer(&res?, &mut buffer)
        .map_err(|e| format!("failed to serialize results: {}", e.to_string()))?;
    Ok(buffer)
}

#[wasm_func]
fn define_vars(ctx_name: &[u8], vars: &[u8], type_field: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_context(ctx_name)?;

    let variables: HashMap<String, value::JSBytesValue> = ciborium::from_reader(vars)
        .map_err(|e| format!("failed to deserialize vars: {}", e.to_string()))?;

    let type_field =
        std::str::from_utf8(type_field).map_err(|e| format!("failed to parse type_field: {}", e.to_string()))?.to_string();

    ctx.with(|ctx| {
        let variables: String = variables
            .into_iter()
            .map::<Result<String, String>, _>(|(k, v)| {
                Ok(format!("let {}={}", k, v.to_value_string(&ctx, &type_field)?))
            })
            .collect::<Result<Vec<String>, String>>()?
            .join(";");

        _ = ctx
            .eval::<rquickjs::Value, std::string::String>(format!("{};", variables))
            .catch(&ctx)
            .map_err(|e| format!("eval error: {}", e.to_string()));
        Ok(vec![])
    })
}

#[wasm_func]
fn eval_format(ctx_name: &[u8], js: &[u8], args: &[u8], type_field: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_context(ctx_name)?;

    let js =
        std::str::from_utf8(js).map_err(|e| format!("failed to parse js: {}", e.to_string()))?;

    let arguments: HashMap<String, value::JSBytesValue> = ciborium::from_reader(args)
        .map_err(|e| format!("failed to deserialize args: {}", e.to_string()))?;

    let type_field =
        std::str::from_utf8(type_field).map_err(|e| format!("failed to parse type_field: {}", e.to_string()))?.to_string();

    let res: Result<JSBytesValue, String> = ctx.with(|ctx| {
        let arguments = arguments
            .into_iter()
            .map::<Result<(String, String), String>, _>(|(k, v)| Ok((k, v.to_value_string(&ctx, &type_field)?)))
            .collect::<Result<HashMap<String, String>, String>>()?;
        let mut options = EvalOptions::default();
        options.global = true;
        ctx.eval_with_options(
            strfmt(js, &arguments).map_err(|e| format!("can not format js string: {}", e))?,
            options,
        )
        .catch(&ctx)
        .map_err(|e| format!("eval error: {}", e.to_string()))
    });

    let mut buffer = vec![];
    _ = ciborium::ser::into_writer(&res?, &mut buffer)
        .map_err(|e| format!("failed to serialize results: {}", e.to_string()))?;
    Ok(buffer)
}

#[wasm_func]
fn call_function(ctx_name: &[u8], fn_name: &[u8], args: &[u8], type_field: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_context(ctx_name)?;

    let fn_name: &str = std::str::from_utf8(fn_name)
        .map_err(|e| format!("failed to parse fn_name: {}", e.to_string()))?;

    let arguments: Vec<value::JSBytesValue> = ciborium::from_reader(args)
        .map_err(|e| format!("failed to deserialize args: {}", e.to_string()))?;

    let type_field =
        std::str::from_utf8(type_field).map_err(|e| format!("failed to parse type_field: {}", e.to_string()))?.to_string();

    let res: Result<JSBytesValue, String> = ctx.with(|ctx| {
        let mut args = Args::new(ctx.clone(), arguments.len());
        for ele in arguments {
            _ = args
                .push_arg(ele.to_js(&ctx, &type_field)?)
                .catch(&ctx)
                .map_err(|e| format!("failed to add arg: {}", e.to_string()))?;
        }

        let func: rquickjs::Function = ctx
            .globals()
            .get(fn_name)
            .catch(&ctx)
            .map_err(|e| format!("failed to get function: {}", e.to_string()))?;

        func.call_arg(args)
            .catch(&ctx)
            .map_err(|e| format!("failed to call function: {}", e.to_string()))
    });

    let mut buffer = vec![];
    _ = ciborium::ser::into_writer(&res?, &mut buffer)
        .map_err(|e| format!("failed to serialize results: {}", e.to_string()))?;
    Ok(buffer)
}

#[wasm_func]
fn compile_module_bytecode(
    ctx_name: &[u8],
    module_name: &[u8],
    module: &[u8],
    compress: &[u8],
) -> Result<Vec<u8>, String> {
    let ctx = get_context(ctx_name)?;

    let module_name: &str = std::str::from_utf8(module_name)
        .map_err(|e| format!("failed to parse module_name: {}", e.to_string()))?;

    let module: &str = std::str::from_utf8(module)
        .map_err(|e| format!("failed to parse module: {}", e.to_string()))?;

    if compress.len() != 1 {
        return Err("failed to parse compress: is not a one sized array".to_string())
    }

    let compress = compress[0] != 0;

    ctx.with(|ctx| {
        let m = Module::declare(ctx, module_name, module)
            .map_err(|e| format!("failed declare module: {}", e.to_string()))?;
        let byte_code = m
            .write(false)
            .map_err(|e| format!("failed to get bytecode: {}", e.to_string()))?;
        if compress {
            return Ok(compress_prepend_size(&byte_code))
        }
        Ok(byte_code)
    })
}

#[wasm_func]
fn load_module_bytecode(ctx_name: &[u8], bytecode: &[u8], compressed: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_context(ctx_name)?;

    if compressed.len() != 1 {
        return Err("failed to parse compress: is not a one sized array".to_string())
    }

    let compressed = compressed[0] != 0;

    if compressed {
        let bytecode = decompress_size_prepended(bytecode).map_err(|e| e.to_string())?;

        ctx.with(|ctx| {
            let m = unsafe { Module::load(ctx.clone(), &bytecode) }
                .catch(&ctx)
                .map_err(|e| format!("failed load bytecode: {}", e.to_string()))?;
            _ = m
                .eval()
                .catch(&ctx)
                .map_err(|e| format!("failed eval bytecode: {}", e.to_string()))?;
    
            Ok(vec![])
        })
    } else {
        ctx.with(|ctx| {
            let m = unsafe { Module::load(ctx.clone(), &bytecode) }
                .catch(&ctx)
                .map_err(|e| format!("failed load bytecode: {}", e.to_string()))?;
            _ = m
                .eval()
                .catch(&ctx)
                .map_err(|e| format!("failed eval bytecode: {}", e.to_string()))?;
    
            Ok(vec![])
        })
    }
}

#[wasm_func]
fn load_module_js(ctx_name: &[u8], module_name: &[u8], module: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_context(ctx_name)?;

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
    ctx_name: &[u8],
    module_name: &[u8],
    fn_name: &[u8],
    args: &[u8],
    type_field: &[u8],
) -> Result<Vec<u8>, String> {
    let ctx = get_context(ctx_name)?;

    let module_name: &str = std::str::from_utf8(module_name)
        .map_err(|e| format!("failed to parse module_name: {}", e.to_string()))?;

    let fn_name: &str = std::str::from_utf8(fn_name)
        .map_err(|e| format!("failed to parse fn_name: {}", e.to_string()))?;

    let arguments: Vec<value::JSBytesValue> = ciborium::from_reader(args)
        .map_err(|e| format!("failed to deserialize args: {}", e.to_string()))?;

    let type_field =
        std::str::from_utf8(type_field).map_err(|e| format!("failed to parse type_field: {}", e.to_string()))?.to_string();

    let res: Result<JSBytesValue, String> = ctx.with(|ctx| {
        let mut args = Args::new(ctx.clone(), arguments.len());
        for ele in arguments {
            _ = args
                .push_arg(ele.to_js(&ctx, &type_field)?)
                .catch(&ctx)
                .map_err(|e| format!("failed to add arg: {}", e.to_string()))?;
        }

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

        func.call_arg(args)
            .catch(&ctx)
            .map_err(|e| format!("failed to call function: {}", e.to_string()))
    });

    let mut buffer = vec![];
    _ = ciborium::ser::into_writer(&res?, &mut buffer)
        .map_err(|e| format!("failed to serialize results: {}", e.to_string()))?;
    Ok(buffer)
}

#[wasm_func]
fn get_module_properties(ctx_name: &[u8], module_name: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_context(ctx_name)?;
    
    let module_name: &str = std::str::from_utf8(module_name)
        .map_err(|e| format!("failed to parse module_name: {}", e.to_string()))?;

    let res: Result<Vec<String>, String> = ctx.with(|ctx| {
        let m: rquickjs::Object = Module::import(&ctx, module_name)
            .catch(&ctx)
            .map_err(|e| format!("failed to import module: {}", e.to_string()))?
            .finish()
            .catch(&ctx)
            .map_err(|e| format!("failed to finish module import: {}", e.to_string()))?;

        m.keys()
            .map(|f| {
                f.catch(&ctx)
                    .map_err(|e| format!("can not collect module keys: {}", e.to_string()))
            })
            .collect::<Result<Vec<String>, String>>()
    });

    let mut buffer = vec![];
    _ = ciborium::ser::into_writer(&res?, &mut buffer)
        .map_err(|e| format!("failed to serialize results: {}", e.to_string()))?;
    Ok(buffer)
}
