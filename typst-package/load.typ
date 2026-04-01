#import "internal.typ" as _internal

#let eval(js) = {
  let value = bytes(js)
  return bytes((_internal.cbor-byte-type, value.len() + 1, _internal.eval)) + value
}

#let eval-format(js, args) = {
  let value = cbor.encode((js, args))
  return bytes((_internal.cbor-byte-type + value.len() + 1, _internal.eval-format)) + value
}

#let define-vars(vars) = {
  let value = cbor.encode((type-field, vars))
  return bytes((_internal.cbor-byte-type + value.len() + 1, _internal.define-vars)) + value
}

#let call-function(fnname, args) = {
  let value = cbor.encode((fnname, args))
  return bytes((_internal.cbor-byte-type + value.len() + 1, _internal.call-function)) + value
}

#let load-module-bytecode(bytecode) = {
  return bytes((_internal.cbor-byte-type + bytecode.len() + 1, _internal.load-module-bytecode)) + bytecode
}

#let load-module-js(modulename, module) = {
  let value = cbor.encode((modulename, module))
  return bytes((_internal.cbor-byte-type + value.len() + 1, _internal.load-module-js)) + value
}

#let call-module-function(modulename, fnname, args) = {
  let value = cbor.encode((modulename, fnname, args))
  return bytes((_internal.cbor-byte-type + value.len() + 1, _internal.call-module-function)) + value
}
