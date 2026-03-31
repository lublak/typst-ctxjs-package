use minicbor::Decoder;
use rquickjs::{
    context::EvalOptions, function::Args, CatchResultExt, CaughtError, Context, Module,
};
use strfmt::strfmt;

use crate::{
    args,
    cbor::{self, con},
};

fn run_load_eval(js: &[u8], ctx: &Context) -> Result<(), minicbor::decode::Error> {
    let js: &str = std::str::from_utf8(js).map_err(|err| minicbor::decode::Error::custom(err))?;

    let mut options = EvalOptions::default();
    options.global = true;

    _ = ctx.with(|ctx| -> Result<(), minicbor::decode::Error> {
        ctx.eval_with_options(js, options)
            .catch(&ctx)
            .map_err(|err: CaughtError| {
                minicbor::decode::Error::message(format!("eval error: {}", err))
            })
    })?;

    Ok(())
}

fn cbor_decode_run_load_eval_format(
    decoder: &mut Decoder,
    ctx: &Context,
) -> Result<(), minicbor::decode::Error> {
    cbor::utils::array_fixed_length(decoder, 2)?;

    let js = decoder.str()?;
    let arguments = args::string_map(decoder)?;

    let mut options = EvalOptions::default();
    options.global = true;

    _ = ctx.with(|ctx| -> Result<(), minicbor::decode::Error> {
        ctx.eval_with_options(
            strfmt(&js, &arguments).map_err(|err| {
                minicbor::decode::Error::message(format!("can not format js string: {}", err))
            })?,
            options,
        )
        .catch(&ctx)
        .map_err(|err: CaughtError| minicbor::decode::Error::message(err))
    })?;
    Ok(())
}

fn cbor_decode_run_load_define_vars(
    decoder: &mut Decoder,
    ctx: &Context,
) -> Result<(), minicbor::decode::Error> {
    let variables = args::string_map(decoder)?
        .into_iter()
        .map(|(k, v)| format!("let {}={}", k, v))
        .fold(String::new(), |a, b| a + &b + ";");

    _ = ctx.with(|ctx| -> Result<(), minicbor::decode::Error> {
        ctx.eval(format!("{};", variables))
            .catch(&ctx)
            .map_err(|err: CaughtError| {
                minicbor::decode::Error::message(format!("eval error: {}", err.to_string()))
            })
    })?;
    Ok(())
}

fn cbor_decode_run_call_function(
    decoder: &mut Decoder,
    ctx: &Context,
) -> Result<(), minicbor::decode::Error> {
    cbor::utils::array_fixed_length(decoder, 2)?;

    let fn_name = decoder.str()?;

    _ = ctx.with(|ctx| -> Result<(), minicbor::decode::Error> {
        let arguments: Vec<rquickjs::Value> = args::array(&ctx, decoder).map_err(|e| {
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

        func.call_arg(args).catch(&ctx).map_err(|e| {
            minicbor::decode::Error::message(format!("failed to call function: {}", e.to_string()))
        })
    })?;

    Ok(())
}

fn run_load_module_byte_code(
    bytecode: &[u8],
    ctx: &Context,
) -> Result<(), minicbor::decode::Error> {
    _ = ctx.with(|ctx| -> Result<(), minicbor::decode::Error> {
        _ = unsafe { Module::load(ctx.clone(), bytecode) }
            .catch(&ctx)
            .map_err(|e| {
                minicbor::decode::Error::message(format!("failed load bytecode: {}", e.to_string()))
            })?
            .eval()
            .catch(&ctx)
            .map_err(|e| {
                minicbor::decode::Error::message(format!("failed eval bytecode: {}", e.to_string()))
            })?;
        Ok(())
    })?;

    Ok(())
}

fn cbor_decode_run_load_module_js(
    decoder: &mut Decoder,
    ctx: &Context,
) -> Result<(), minicbor::decode::Error> {
    cbor::utils::array_fixed_length(decoder, 2)?;

    let module_name = decoder.str()?;
    let module_code = decoder.bytes()?;

    _ = ctx.with(|ctx| -> Result<(), minicbor::decode::Error> {
        Module::declare(ctx.clone(), module_name, module_code)
            .catch(&ctx)
            .map_err(|e| {
                minicbor::decode::Error::message(format!(
                    "failed load module code: {}",
                    e.to_string()
                ))
            })?
            .eval()
            .catch(&ctx)
            .map_err(|e| {
                minicbor::decode::Error::message(format!(
                    "failed eval module code: {}",
                    e.to_string()
                ))
            })?;
        Ok(())
    })?;

    Ok(())
}

fn cbor_decode_run_call_module_function(
    decoder: &mut Decoder,
    ctx: &Context,
) -> Result<(), minicbor::decode::Error> {
    cbor::utils::array_fixed_length(decoder, 3)?;

    let module_name = decoder.str()?;
    let fn_name = decoder.str()?;

    _ = ctx.with(|ctx| -> Result<(), minicbor::decode::Error> {
        let arguments: Vec<rquickjs::Value> = args::array(&ctx, decoder).map_err(|e| {
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
                minicbor::decode::Error::message(format!(
                    "failed to import module: {}",
                    e.to_string()
                ))
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

        func.call_arg(args).catch(&ctx).map_err(|e| {
            minicbor::decode::Error::message(format!("failed to call function: {}", e.to_string()))
        })
    })?;

    Ok(())
}

pub(crate) fn cbor_decode_run_load(
    decoder: &mut Decoder,
    ctx: &Context,
) -> Result<(), minicbor::decode::Error> {
    for _ in 0..cbor::utils::array_length(decoder)? {
        let b = decoder.bytes()?;
        if let Some(h) = b.get(0) {
            match h {
                &con::EVAL => {
                    run_load_eval(&b[1..], ctx)?;
                }
                &con::EVAL_FORMAT => {
                    cbor_decode_run_load_eval_format(&mut Decoder::new(&b[1..]), ctx)?;
                }
                &con::DEFINE_VARS => {
                    cbor_decode_run_load_define_vars(&mut Decoder::new(&b[1..]), ctx)?;
                }
                &con::CALL_FUNCTION => {
                    cbor_decode_run_call_function(&mut Decoder::new(&b[1..]), ctx)?;
                }
                &con::LOAD_MODULE_BYTECODE => {
                    run_load_module_byte_code(&b[1..], ctx)?;
                }
                &con::LOAD_MODULE_JS => {
                    cbor_decode_run_load_module_js(&mut Decoder::new(&b[1..]), ctx)?;
                }
                &con::CALL_MODULE_FUNCTION => {
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
