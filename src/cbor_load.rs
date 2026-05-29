use minicbor::Decoder;
use rquickjs::Ctx;
use rquickjs::{context::EvalOptions, function::Args, CatchResultExt, CaughtError, Module};

use crate::cbor;
use crate::strfmt;

const LOAD_EVAL: u8 = 0;
const LOAD_EVAL_FORMAT: u8 = 1;
const LOAD_DEFINE_VARS: u8 = 2;
const LOAD_CALL_FUNCTION: u8 = 3;
const LOAD_LOAD_MODULE_BYTECODE: u8 = 4;
const LOAD_LOAD_MODULE_JS: u8 = 5;
const LOAD_CALL_MODULE_FUNCTION: u8 = 6;

fn cbor_decode_run_load_eval<'js>(
    decoder: &mut Decoder,
    ctx: &Ctx<'js>,
) -> cbor::rquickjs::decode::Result<'js, ()> {
    let js = decoder.bytes()?;

    let mut options = EvalOptions::default();
    options.global = true;

    let _: rquickjs::Value = ctx.eval_with_options(js, options).catch(&ctx)?;

    Ok(())
}

fn cbor_decode_run_load_eval_format<'js>(
    decoder: &mut Decoder,
    ctx: &Ctx<'js>,
) -> cbor::rquickjs::decode::Result<'js, ()> {
    cbor::utils::array_fixed_length(decoder, 2)?;

    let js = decoder.bytes()?;
    let arguments = cbor::rquickjs::args::string_map(decoder)?;

    let mut options = EvalOptions::default();
    options.global = true;

    let _: rquickjs::Value = ctx
        .eval_with_options(
            strfmt::strfmt(&js, &arguments).map_err(|err| {
                minicbor::decode::Error::message(format!("can not format js string: {}", err))
            })?,
            options,
        )
        .catch(&ctx)?;

    Ok(())
}

fn cbor_decode_run_load_define_vars<'js>(
    decoder: &mut Decoder,
    ctx: &Ctx<'js>,
) -> cbor::rquickjs::decode::Result<'js, ()> {
    let variables = cbor::rquickjs::args::string_map(decoder)?
        .into_iter()
        .map(|(k, v)| format!("let {}={}", k, v))
        .fold(String::new(), |a, b| a + &b + ";");

    let _: rquickjs::Value =
        ctx.eval(format!("{};", variables))
            .catch(&ctx)
            .map_err(|err: CaughtError| {
                minicbor::decode::Error::message(format!("eval error: {}", err.to_string()))
            })?;
    Ok(())
}

fn cbor_decode_run_call_function<'js>(
    decoder: &mut Decoder,
    ctx: &Ctx<'js>,
) -> cbor::rquickjs::decode::Result<'js, ()> {
    cbor::utils::array_fixed_length(decoder, 2)?;

    let fn_name = decoder.str()?;

    let arguments: Vec<rquickjs::Value> =
        cbor::rquickjs::args::array(&ctx, decoder).map_err(|e| {
            minicbor::decode::Error::message(format!(
                "failed to deserialize arguments: {}",
                e.to_string()
            ))
        })?;

    let mut args = Args::new(ctx.clone(), arguments.len());
    args.push_args(arguments).map_err(|e| {
        minicbor::decode::Error::message(format!("failed to add args: {}", e.to_string()))
    })?;

    let func: rquickjs::Function = ctx.globals().get(fn_name).catch(&ctx).map_err(|e| {
        minicbor::decode::Error::message(format!("failed to get function: {}", e.to_string()))
    })?;

    let _: rquickjs::Value = func.call_arg(args).catch(&ctx).map_err(|e| {
        minicbor::decode::Error::message(format!("failed to call function: {}", e.to_string()))
    })?;

    Ok(())
}

fn run_load_module_byte_code<'js>(
    bytecode: &[u8],
    ctx: &Ctx<'js>,
) -> cbor::rquickjs::decode::Result<'js, ()> {
    let m = unsafe { Module::load(ctx.clone(), bytecode) }.catch(&ctx)?;
    _ = m.eval().catch(&ctx)?;

    Ok(())
}

fn cbor_decode_run_load_module_js<'js>(
    decoder: &mut Decoder,
    ctx: &Ctx<'js>,
) -> cbor::rquickjs::decode::Result<'js, ()> {
    cbor::utils::array_fixed_length(decoder, 2)?;

    let module_name = decoder.str()?;
    let module_code = decoder.bytes()?;

    let m = Module::declare(ctx.clone(), module_name, module_code).catch(&ctx)?;

    _ = m.eval().catch(&ctx)?;

    Ok(())
}

fn cbor_decode_run_call_module_function<'js>(
    decoder: &mut Decoder,
    ctx: &Ctx<'js>,
) -> cbor::rquickjs::decode::Result<'js, ()> {
    cbor::utils::array_fixed_length(decoder, 3)?;

    let module_name = decoder.str()?;
    let fn_name = decoder.str()?;

    let arguments: Vec<rquickjs::Value> =
        cbor::rquickjs::args::array(&ctx, decoder).map_err(|e| {
            minicbor::decode::Error::message(format!(
                "failed to deserialize arguments: {}",
                e.to_string()
            ))
        })?;

    let mut args = Args::new(ctx.clone(), arguments.len());
    args.push_args(arguments).map_err(|e| {
        minicbor::decode::Error::message(format!("failed to add args: {}", e.to_string()))
    })?;

    let m: rquickjs::Object = Module::import(&ctx, module_name)
        .catch(&ctx)
        .map_err(|e| {
            minicbor::decode::Error::message(format!("failed to import module: {}", e.to_string()))
        })?
        .finish()
        .catch(&ctx)
        .map_err(|e| {
            minicbor::decode::Error::message(format!(
                "failed to finish module import: {}",
                e.to_string()
            ))
        })?;

    let func: rquickjs::Function = m.get(fn_name).catch(&ctx).map_err(|e| {
        minicbor::decode::Error::message(format!("failed to get function: {}", e.to_string()))
    })?;

    let _: rquickjs::Value = func.call_arg(args).catch(&ctx).map_err(|e| {
        minicbor::decode::Error::message(format!("failed to call function: {}", e.to_string()))
    })?;

    Ok(())
}

pub(crate) fn cbor_decode_run_load<'js>(
    decoder: &mut Decoder,
    ctx: &Ctx<'js>,
) -> cbor::rquickjs::decode::Result<'js, ()> {
    for _ in 0..cbor::utils::array_length(decoder)? {
        let b = decoder.bytes()?;
        if let Some(h) = b.get(0) {
            match h {
                &LOAD_EVAL => {
                    cbor_decode_run_load_eval(&mut Decoder::new(&b[1..]), ctx)?;
                }
                &LOAD_EVAL_FORMAT => {
                    cbor_decode_run_load_eval_format(&mut Decoder::new(&b[1..]), ctx)?;
                }
                &LOAD_DEFINE_VARS => {
                    cbor_decode_run_load_define_vars(&mut Decoder::new(&b[1..]), ctx)?;
                }
                &LOAD_CALL_FUNCTION => {
                    cbor_decode_run_call_function(&mut Decoder::new(&b[1..]), ctx)?;
                }
                &LOAD_LOAD_MODULE_BYTECODE => {
                    run_load_module_byte_code(&b[1..], ctx)?;
                }
                &LOAD_LOAD_MODULE_JS => {
                    cbor_decode_run_load_module_js(&mut Decoder::new(&b[1..]), ctx)?;
                }
                &LOAD_CALL_MODULE_FUNCTION => {
                    cbor_decode_run_call_module_function(&mut Decoder::new(&b[1..]), ctx)?;
                }
                _ => Err(minicbor::decode::Error::message(format!(
                    "unsupported header {}",
                    h
                )))?,
            }
        }
    }

    Ok(())
}
