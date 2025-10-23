#let ctxjs = plugin("ctxjs.wasm")

#import "load.typ" as load
#import "ctx.typ" as ctx

#let new-context(load: ()) = {
  return plugin.transition(ctxjs.new_context, cbor.encode(load))
}

#let image-data-url(data, format: auto) = {
  if type(data) == str {
    data = read(data, encoding: none)
  }
  if format == auto {
    format = ""
  }
  return str(ctxjs.image_data_url(data))
}
