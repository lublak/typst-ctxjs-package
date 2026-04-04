# CtxJS

## Description

A typst plugin to evaluate javascript code.

- multiple javascript contexts
- load javascript modules as source or bytecode
- simple evaluations
- formated evaluations (execute your code with your typst data)
- call functions
- call functions in modules
- create quickjs bytecode with an extra tool, to improve loading performance (ctxjs_module_bytecode_builder)
- allow later evaluation of javascript code
- allow loading json directly
- convert images to data urls

## Documentation

A full documentation can be found here: [docs.pdf](https://raw.githubusercontent.com/lublak/typst-ctxjs-package/refs/heads/main/docs.pdf)

## An actively used package

To get a picture what is possible with ctxjs there is a package based on echarts embedded into typst.
It uses a custom js module code to wrap the echarts code in a single function.
The package uses ctxjs_module_bytecode_builder to build the js module code into bytecode.
And it get loaded by typst into a context and the js function gets called.
Which than returns an svg which can be used on the typst side.

[Echarm](https://github.com/lublak/typst-echarm-package)