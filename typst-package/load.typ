#import "helpers.typ"

#let eval-later = helpers.eval-later

#let eval(js) = {
  return ("eval", js)
}

#let eval-format(js, args, type-field: "$type") = {
  return ("eval_format", (js, type-field, args))
}

#let define-vars(vars, type-field: "$type") = {
  return ("define_vars", (type-field, vars))
}

#let call-function(fnname, args, type-field: "$type") = {
  return ("call_function", (fnname, type-field, args))
}

#let load-module-bytecode(bytecode) = {
  return ("load_module_bytecode", bytecode)
}

#let load-module-js(modulename, module) = {
  return ("load_module_js", (modulename, module))
}

#let call-module-function(modulename, fnname, args, type-field: "$type") = {
  return ("call_module_function", (modulename, fnname, type-field, args))
}
