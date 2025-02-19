#let eval(js) = {
  return (ctxjs) => {
    cbor(ctxjs.eval(string-to-bytes(js)))
  }
}

#let call-function(fnname, args, type-field: "$type") = {
  return (ctxjs) => {
    cbor(ctxjs.call_function(string-to-bytes(fnname), cbor.encode(args), string-to-bytes(type-field)))
  }
}

#let define-vars(vars, type-field: "$type") = {
  return (ctxjs) => {
    cbor(ctxjs.define_vars(cbor.encode(vars), string-to-bytes(type-field)))
  }
}

#let eval-format(js, args, type-field: "$type") = {
  return (ctxjs) => {
    cbor(ctxjs.eval_format(string-to-bytes(js), cbor.encode(args), string-to-bytes(type-field)))
  }
}

#let load-module-bytecode(bytecode) = {
  return (ctxjs) => {
    ctxjs.load_module_bytecode(bytecode)
  }
}

#let load-module-js(modulename, module) = {
  return (ctxjs) => {
    ctxjs.load_module_js(string-to-bytes(modulename), string-to-bytes(module))
  }
}

#let call-module-function(modulename, fnname, args, type-field: "$type") = {
  return (ctxjs) => {
    cbor(ctxjs.call_module_function(string-to-bytes(modulename), string-to-bytes(fnname), cbor.encode(args), string-to-bytes(type-field)))
  }
}

#let get-module-properties(modulename) = {
  return (ctxjs) => {
    cbor(ctxjs.get_module_properties(string-to-bytes(modulename)))
  }
}