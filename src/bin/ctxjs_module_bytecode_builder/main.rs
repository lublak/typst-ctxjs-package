use rquickjs::{Context, Module, Runtime};
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        panic!("missing input output arguments, example: ctxjs_module_bytecode_builder name input.js output.")
    }

    let name = &args[1];
    let input = &args[2];
    let output = &args[3];

    let source = fs::read_to_string(input).unwrap();

    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();
    let byte_code = ctx.with(|ctx| {
        let m = Module::declare(ctx, name.as_str(), source).unwrap();
        let byte_code = m.write(false).unwrap();
        byte_code
    });
    fs::write(output, byte_code).unwrap();
}
