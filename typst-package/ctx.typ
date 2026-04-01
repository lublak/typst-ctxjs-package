#import "helpers.typ"

#let cbor-array-type = 128

#let eval-later = helpers.eval-later

#let json = helpers.json

#let image-data-url = helpers.image-data-url

#let load(ctx, ..load) = {
  let args = load.pos()
  let data = bytes((cbor-array-type + args.len(),))
  for value in args {
    data = data + value
  }
  (
    ctx: plugin.transition(ctx.load, data),
    value: none,
  )
}

#let eval(ctx, js, transition: false) = {
  helpers.transition-call(
    ctx,
    ctx.eval,
    transition,
    helpers.string-to-bytes(js),
  )
}

#let eval-format(ctx, js, args, transition: false) = {
  helpers.transition-call(
    ctx,
    ctx.eval_format,
    transition,
    helpers.string-to-bytes(js),
    cbor.encode(args),
  )
}

#let define-vars(ctx, vars, transition: false) = {
  helpers.transition-call(
    ctx,
    ctx.define_vars,
    transition,
    cbor.encode(vars),
  )
}

#let call-function(ctx, fnname, args, transition: false) = {
  helpers.transition-call(
    ctx,
    ctx.call_function,
    transition,
    helpers.string-to-bytes(fnname),
    cbor.encode(args),
    helpers.string-to-bytes(type-field),
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
    ctx: plugin.transition(ctx.load_module_js, helpers.string-to-bytes(modulename), helpers.string-to-bytes(module)),
    value: none,
  )
}

#let call-module-function(ctx, modulename, fnname, args, transition: false) = {
  helpers.transition-call(
    ctx,
    ctx.call_module_function,
    transition,
    helpers.string-to-bytes(modulename),
    helpers.string-to-bytes(fnname),
    cbor.encode(args),
  )
}

#let get-module-properties(ctx, modulename) = {
  (
    ctx: ctx,
    value: cbor(ctx.get_module_properties(helpers.string-to-bytes(modulename))),
  )
}
