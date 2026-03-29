#import "helpers.typ"
#import "con.typ"

#let cbor-byte-type = 64

#let eval(js) = {
  let value = bytes(js)
  return bytes((cbor-byte-type, value.len() + 1, con.eval)) + value
}

#let eval-format(js, args) = {
  let value = cbor.encode((js, args))
  return bytes((cbor-byte-type + value.len() + 1, con.eval-format)) + value
}

#let define-vars(vars) = {
  let value = cbor.encode((type-field, vars))
  return bytes((cbor-byte-type + value.len() + 1, con.define-vars)) + value
}

#let call-function(fnname, args) = {
  let value = cbor.encode((fnname, args))
  return bytes((cbor-byte-type + value.len() + 1, con.call-function)) + value
}

#let load-module-bytecode(bytecode) = {
  return bytes((cbor-byte-type + bytecode.len() + 1, con.load-module-bytecode)) + bytecode
}

#let load-module-js(modulename, module) = {
  let value = cbor.encode((modulename, module))
  return bytes((cbor-byte-type + value.len() + 1, con.load-module-js)) + value
}

#let call-module-function(modulename, fnname, args) = {
  let value = cbor.encode((modulename, fnname, args))
  return bytes((cbor-byte-type + value.len() + 1, con.call-module-function)) + value
}
