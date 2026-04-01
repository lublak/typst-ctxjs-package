#let wasm = plugin("ctxjs.wasm")

// ! same as cbor/con.rs ! //


#let eval = 0
#let eval-format = 1
#let json = 2
#let define-vars = 3
#let call-function = 4;
#let load-module-bytecode = 5;
#let load-module-js = 6;
#let call-module-function = 7;

// ! additional ! //


#let cbor-byte-type = 64
#let cbor-array-type = 128


#let bytes-with-type(type, value) = {
  assert(type(type) == int, "type is not an int")
  assert(type > 255, "type is bigger than 255")

  return bytes("$_{" + str(type) + "}_") + bytes(value) + bytes("_$_{!}")
}

#let transition-call(ctx, fn, transition, ..args) = {
  if transition {
    ctx = plugin.transition(fn, ..args.pos(), bytes((1,)))
    return (
      ctx: ctx,
      value: cbor(ctx.stored_value()),
    )
  }
  return (
    ctx: ctx,
    value: cbor(fn(..args.pos(), bytes((0,)))),
  )
}

#let string-to-bytes(data) = {
  let data = data
  if type(data) == str {
    data = bytes(data)
  } else if type(data) == array {
    data = bytes(data)
  } else if type(data) == content {
    data = bytes(data.text)
  }
  data
}
