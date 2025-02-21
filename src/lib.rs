use std::collections::HashMap;

use rquickjs::{context::EvalOptions, function::Args, CatchResultExt, Context, Module, Runtime};
use value::JSBytesValue;
use wasm_minimal_protocol::*;

use strfmt::strfmt;

mod load;
mod value;

initiate_protocol!();

struct ContextHolder(Option<Context>);
unsafe impl Send for ContextHolder {}
unsafe impl Sync for ContextHolder {}

static mut CURRENT_CONTEXT: ContextHolder = ContextHolder(None);

#[wasm_func]
fn new_context(load: &[u8]) -> Result<Vec<u8>, String> {
    let runtime =
        Runtime::new().map_err(|e| format!("failed to create runtime: {}", e.to_string()))?;

    let ctx: Context = Context::full(&runtime)
        .map_err(|e| format!("failed to create context: {}", e.to_string()))?;

    run_load(&ctx, load)?;

    unsafe {
        CURRENT_CONTEXT.0 = Some(ctx);
    }

    Ok(vec![])
}

#[inline(always)]
#[allow(static_mut_refs)]
fn get_current_context() -> Result<Context, String> {
    return Ok(unsafe { CURRENT_CONTEXT.0.to_owned().ok_or_else(|| "context empty") }?);
}

fn run_load(ctx: &Context, load: &[u8]) -> Result<Vec<u8>, String> {
    let load: Vec<load::Method> = ciborium::from_reader(load)
        .map_err(|e| format!("failed to deserialize load: {}", e.to_string()))?;

    for ele in load {
        match ele {
            load::Method::Eval(js) => {
                let res: Result<JSBytesValue, String> = ctx.with(|ctx| {
                    let mut options = EvalOptions::default();
                    options.global = true;
                    ctx.eval_with_options(js, options)
                        .catch(&ctx)
                        .map_err(|e| format!("eval error: {}", e.to_string()))
                });
                if let Err(err) = res {
                    return Err(err);
                }
            }
            load::Method::EvalFormat(js, arguments, type_field) => {
                let res: Result<JSBytesValue, String> = ctx.with(|ctx| {
                    let arguments = arguments
                        .into_iter()
                        .map::<Result<(String, String), String>, _>(|(k, v)| {
                            Ok((k, v.to_value_string(&ctx, &type_field)?))
                        })
                        .collect::<Result<HashMap<String, String>, String>>()?;
                    let mut options = EvalOptions::default();
                    options.global = true;
                    ctx.eval_with_options(
                        strfmt(&js, &arguments)
                            .map_err(|e| format!("can not format js string: {}", e))?,
                        options,
                    )
                    .catch(&ctx)
                    .map_err(|e| format!("eval error: {}", e.to_string()))
                });
                if let Err(err) = res {
                    return Err(err);
                }
            }
            load::Method::DefineVars(variables, type_field) => {
                _ = ctx.with(|ctx| -> Result<Vec<u8>, String> {
                    let variables: String = variables
                        .into_iter()
                        .map::<Result<String, String>, _>(|(k, v)| {
                            Ok(format!(
                                "let {}={}",
                                k,
                                v.to_value_string(&ctx, &type_field)?
                            ))
                        })
                        .collect::<Result<Vec<String>, String>>()?
                        .join(";");

                    _ = ctx
                        .eval::<rquickjs::Value, std::string::String>(format!("{};", variables))
                        .catch(&ctx)
                        .map_err(|e| format!("eval error: {}", e.to_string()));
                    Ok(vec![])
                })?;
            }
            load::Method::CallFunction(fn_name, arguments, type_field) => {
                _ = ctx.with(|ctx| -> Result<Vec<u8>, String> {
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
                })?;
            }
            load::Method::LoadModuleBytecode(bytecode) => {
                _ = ctx.with(|ctx| -> Result<Vec<u8>, String> {
                    let m = unsafe { Module::load(ctx.clone(), &bytecode) }
                        .catch(&ctx)
                        .map_err(|e| format!("failed load bytecode: {}", e.to_string()))?;
                    _ = m
                        .eval()
                        .catch(&ctx)
                        .map_err(|e| format!("failed eval bytecode: {}", e.to_string()))?;

                    Ok(vec![])
                })?;
            }
            load::Method::LoadModuleJs(module_name, module) => {
                _ = ctx.with(|ctx| -> Result<Vec<u8>, String> {
                    _ = Module::declare(ctx.clone(), module_name, module)
                        .catch(&ctx)
                        .map_err(|e| format!("failed load module code: {}", e.to_string()))?
                        .eval()
                        .catch(&ctx)
                        .map_err(|e| format!("failed eval module code: {}", e.to_string()))?;
                    Ok(vec![])
                })?;
            }
            load::Method::CallModuleFunction(module_name, fn_name, arguments, type_field) => {
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
                        .map_err(|e| {
                            format!("failed to finish module import: {}", e.to_string())
                        })?;

                    let func: rquickjs::Function = m
                        .get(fn_name)
                        .catch(&ctx)
                        .map_err(|e| format!("failed to get function: {}", e.to_string()))?;

                    func.call_arg(args)
                        .catch(&ctx)
                        .map_err(|e| format!("failed to call function: {}", e.to_string()))
                });

                if let Err(err) = res {
                    return Err(err);
                }
            }
        }
    }

    Ok(vec![])
}

#[wasm_func]
fn load(run: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    run_load(&ctx, run)?;

    Ok(vec![])
}

#[wasm_func]
fn eval(js: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

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
fn eval_format(js: &[u8], arguments: &[u8], type_field: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let js =
        std::str::from_utf8(js).map_err(|e| format!("failed to parse js: {}", e.to_string()))?;

    let arguments: HashMap<String, value::JSBytesValue> = ciborium::from_reader(arguments)
        .map_err(|e| format!("failed to deserialize arguments: {}", e.to_string()))?;

    let type_field = std::str::from_utf8(type_field)
        .map_err(|e| format!("failed to parse type_field: {}", e.to_string()))?
        .to_string();

    let res: Result<JSBytesValue, String> = ctx.with(|ctx| {
        let arguments = arguments
            .into_iter()
            .map::<Result<(String, String), String>, _>(|(k, v)| {
                Ok((k, v.to_value_string(&ctx, &type_field)?))
            })
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
fn define_vars(variables: &[u8], type_field: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let variables: HashMap<String, value::JSBytesValue> = ciborium::from_reader(variables)
        .map_err(|e| format!("failed to deserialize variables: {}", e.to_string()))?;

    let type_field = std::str::from_utf8(type_field)
        .map_err(|e| format!("failed to parse type_field: {}", e.to_string()))?
        .to_string();

    ctx.with(|ctx| {
        let variables: String = variables
            .into_iter()
            .map::<Result<String, String>, _>(|(k, v)| {
                Ok(format!(
                    "let {}={}",
                    k,
                    v.to_value_string(&ctx, &type_field)?
                ))
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
fn call_function(fn_name: &[u8], arguments: &[u8], type_field: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let fn_name: &str = std::str::from_utf8(fn_name)
        .map_err(|e| format!("failed to parse fn_name: {}", e.to_string()))?;

    let arguments: Vec<value::JSBytesValue> = ciborium::from_reader(arguments)
        .map_err(|e| format!("failed to deserialize arguments: {}", e.to_string()))?;

    let type_field = std::str::from_utf8(type_field)
        .map_err(|e| format!("failed to parse type_field: {}", e.to_string()))?
        .to_string();

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
fn compile_module_bytecode(module_name: &[u8], module: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let module_name: &str = std::str::from_utf8(module_name)
        .map_err(|e| format!("failed to parse module_name: {}", e.to_string()))?;

    let module: &str = std::str::from_utf8(module)
        .map_err(|e| format!("failed to parse module: {}", e.to_string()))?;

    ctx.with(|ctx| {
        let m = Module::declare(ctx, module_name, module)
            .map_err(|e| format!("failed declare module: {}", e.to_string()))?;
        let byte_code = m
            .write(false)
            .map_err(|e| format!("failed to get bytecode: {}", e.to_string()))?;
        Ok(byte_code)
    })
}

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
    type_field: &[u8],
) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

    let module_name: &str = std::str::from_utf8(module_name)
        .map_err(|e| format!("failed to parse module_name: {}", e.to_string()))?;

    let fn_name: &str = std::str::from_utf8(fn_name)
        .map_err(|e| format!("failed to parse fn_name: {}", e.to_string()))?;

    let arguments: Vec<value::JSBytesValue> = ciborium::from_reader(arguments)
        .map_err(|e| format!("failed to deserialize arguments: {}", e.to_string()))?;

    let type_field = std::str::from_utf8(type_field)
        .map_err(|e| format!("failed to parse type_field: {}", e.to_string()))?
        .to_string();

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
fn get_module_properties(module_name: &[u8]) -> Result<Vec<u8>, String> {
    let ctx = get_current_context()?;

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
