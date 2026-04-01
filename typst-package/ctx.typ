#import "internal.typ" as _internal

#let load(ctx, ..load) = {
  let args = load.pos()
  let data = bytes((_internal.cbor-array-type + args.len(),))
  for value in args {
    data = data + value
  }
  (
    ctx: plugin.transition(ctx.load, data),
    value: none,
  )
}

#let eval(ctx, js, transition: false) = {
  _internal.transition-call(
    ctx,
    ctx.eval,
    transition,
    _internal.string-to-bytes(js),
  )
}

#let eval-format(ctx, js, args, transition: false) = {
  _internal.transition-call(
    ctx,
    ctx.eval_format,
    transition,
    _internal.string-to-bytes(js),
    cbor.encode(args),
  )
}

#let define-vars(ctx, vars, transition: false) = {
  _internal.transition-call(
    ctx,
    ctx.define_vars,
    transition,
    cbor.encode(vars),
  )
}

#let call-function(ctx, fnname, args, transition: false) = {
  _internal.transition-call(
    ctx,
    ctx.call_function,
    transition,
    _internal.string-to-bytes(fnname),
    cbor.encode(args),
    _internal.string-to-bytes(type-field),
  )
}

#let load-module-bytecode(ctx, bytecode) = {
  (
    ctx: plugin.transition(ctx.load_module_bytecode, bytecode),
    value: none,
  )
}

#let load-module-js(ctx, modulename, module) = {
  (
    ctx: plugin.transition(
      ctx.load_module_js,
      _internal.string-to-bytes(modulename),
      _internal.string-to-bytes(module),
    ),
    value: none,
  )
}

#let call-module-function(ctx, modulename, fnname, args, transition: false) = {
  _internal.transition-call(
    ctx,
    ctx.call_module_function,
    transition,
    _internal.string-to-bytes(modulename),
    _internal.string-to-bytes(fnname),
    cbor.encode(args),
  )
}

#let get-module-properties(ctx, modulename) = {
  (
    ctx: ctx,
    value: cbor(ctx.get_module_properties(_internal.string-to-bytes(modulename))),
  )
}
