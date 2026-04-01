#import "internal.typ" as _internal

#import "load.typ" as load
#import "ctx.typ" as ctx
#import "value.typ" as value

#let new-context(..load) = {
  return plugin.transition(_internal.wasm.new_context, cbor.encode(load.pos()))
}
