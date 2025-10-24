#let ctxjs = plugin("ctxjs.wasm")

#import "load.typ" as load
#import "ctx.typ" as ctx

#let new-context(load: ()) = {
  if type(load) != array {
    load = (load,)
  }
  return plugin.transition(ctxjs.new_context, cbor.encode(load))
}

#let image-data-url = ctx.image-data-url
