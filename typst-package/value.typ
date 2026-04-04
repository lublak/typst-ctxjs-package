#import "internal.typ" as _internal

/// Typst *does not* support a js function as a returning value: `#ctxjs.ctx.eval(ctx, "function() {}")`.
/// So as an alternative `ctxjs.value.eval` can be used,
/// which returns a special formated bytes value with raw js code.
///
/// `#ctxjs.ctx.call-function(ctx, "fnname", (ctxjs.value.eval("function(args) { return true; }"),))`
///
/// translates on the js side to
///
/// `fnname(function(args){ return true; })`
/// ```examplec
/// ctxjs.value.eval("1+2")
/// ```
/// -> bytes
#let eval(
  /// the js code which should be evaluate
  /// -> str | bytes
  js,
) = {
  _internal.cbor-tagged-data(_internal.eval, cbor.encode(bytes(js)))
}

/// Similar to @eval the function returns a special formated bytes value with raw js code.
/// Additional it supports formatting of the eval code with typst values.
/// ```examplec
/// ctxjs.value.eval-format("{val1}+{val2}", val1: 1, val2: 2)
/// ```
/// -> bytes
#let eval-format(
  /// the js code which should be evaluate
  /// -> str | bytes
  js,
  /// named args which replaces the name in the js code with the typst value as js value, only characters a-zA-Z0-9\_- as name are allowed
  /// -> any
  ..args,
) = {
  _internal.cbor-tagged-data(_internal.eval-format, cbor.encode((js, args.named())))
}

/// Similar to @eval the function returns a special formated bytes value but with raw json code which will be also validated as pure json code in ctxjs.
/// ```examplec
/// ctxjs.value.json("{}")
/// ```
/// -> bytes
#let json(
  /// json code
  /// -> str | bytes
  json,
) = {
  _internal.cbor-tagged-data(_internal.json, cbor.encode(bytes(json)))
}

/// Returns a data url from an image.
/// ```examplec
/// ctxjs.value.image-data-url(bytes("<svg></svg>"))
/// ```
/// -> str
#let image-data-url(
  /// the png, jpg, gif or svg image as bytes
  /// -> bytes
  data,
  /// can be used to override and pre set the image format, if format is `auto` the format with automatically detected
  /// -> bytes
  format: auto,
) = {
  if format == auto {
    format = ""
  }
  return str(_internal.wasm.image_data_url(data, bytes(format)))
}
