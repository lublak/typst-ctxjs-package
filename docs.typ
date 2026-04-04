#import "typst-package/lib.typ" as ctxjs
#import "@preview/tidy:0.4.3"
#import ctxjs.load
#import ctxjs.ctx
#import ctxjs.value
#let current-context = ctxjs.new-context()

#outline(
  title: [Documentatation],
  depth: 3,
)
#show ref: it => {
  if it.element != none and it.element.func() == heading {
    link(it.element.location(), [
      #tidy.styles.default.show-reference(it.target, str(it.target))
    ])
  } else {
    let strtarget = str(it.target)
    if strtarget.starts-with(regex("(ctxjs|ctx|eval|value|load)\.")) {
      strtarget = strtarget + "()"
      tidy.styles.default.show-reference(label(strtarget), strtarget)
    } else {
      it
    }
  }
}

#let docs = tidy.parse-module(
  read("typst-package/lib.typ"),
  name: "Ctxjs",
  scope: (
    ctxjs: ctxjs,
    current-context: current-context,
  ),
  label-prefix: "ctxjs.",
)
#tidy.show-module(docs, style: tidy.styles.default, first-heading-level: 2)
#pagebreak()
#let docs = tidy.parse-module(
  read("typst-package/ctx.typ"),
  name: "Ctx",
  scope: (
    ctxjs: ctxjs,
    current-context: current-context,
  ),
  label-prefix: "ctx.",
)
#tidy.show-module(docs, style: tidy.styles.default, first-heading-level: 2)
#pagebreak()
#let docs = tidy.parse-module(
  read("typst-package/load.typ"),
  name: "Load",
  scope: (
    ctxjs: ctxjs,
    current-context: current-context,
  ),
  label-prefix: "load.",
)
#tidy.show-module(docs, style: tidy.styles.default, first-heading-level: 2)
#pagebreak()
#let docs = tidy.parse-module(
  read("typst-package/value.typ"),
  name: "Value",
  scope: (
    ctxjs: ctxjs,
    current-context: current-context,
  ),
  label-prefix: "value.",
)
#tidy.show-module(docs, style: tidy.styles.default, first-heading-level: 2)
#pagebreak()

== ctxjs_module_bytecode_builder <ctxjs_module_bytecode_builder>

Can be simple called like this:

```
cargo run --manifest-path typst-ctxjs-package/Cargo.toml --bin ctxjs_module_bytecode_builder modulename js/dist/modulecode.js typst-package/bytecode.kbc1
```

It uses the modulename to compile the module code and write the bytecode to the kbc1 file.

== Guide

#let mainjs = read("example/main.js")
#let current-context = ctxjs.new-context(
  load.eval(mainjs),
  load.eval("let counter = 0; function changes_data() { counter++; return counter; }"),
  load.eval("function call_callback(callback) { return \"Hello \" + callback(); }"),
  load.eval("function pure_json(json) { return json.arr; }"),
)

#show: tidy.render-examples.with(
  scope: (
    ctxjs: ctxjs,
    load: load,
    ctx: ctx,
    value: value,
    current-context: current-context,
    data: (
      logo: read("example/Typst.svg.png", encoding: none),
    ),
  ),
  layout: (code, preview, ..options) => {
    block(
      breakable: false,
      tidy.show-example.default-layout-example(code, preview, ..options),
    )
  },
)

=== example files

- `example/main.js`\
  #raw(mainjs, lang: "js")
- `example/Typst.svg.png`\
  #image("example/Typst.svg.png", width: 25%)

=== new context

Creates a new context to work with and preloaded with a simple evaluated js file and a function.
It is recommend to build your own js file to an esm file and create bytecode from it via ctxjs_module_bytecode_builder.

```typ
#import "@preview/ctxjs:0.4.0"
#import ctxjs.load
#import ctxjs.ctx
#import ctxjs.value

#let current-context = ctxjs.new-context(
  load.eval(read("examples/main.js", encoding: none)),
  load.eval("let counter = 0; function changes_data() { counter++; return counter; }"),
  load.eval("function call_callback(callback) { return "Hello " + callback(); }"),
  load.eval("function pure_json(json) { return json.arr; }"),
  // load.load-module-bytecode(read("main.kbc1", encoding: none)),
)
```

== working with context

On calling any ctx function a new or the current context gets returned with a value as an array.
Its recommend always use the destructuring syntax to be safe that we are always use the correct context.
```example
#let (current-context, value) = ctx.eval(current-context, "123")
#value
```
If you do not need the value, its safe to ignore it:
```example
#let (current-context, _) = ctx.eval(current-context, "123")
value gets ignored
```

=== image-data-url

Calls a javascript which takes a base64 image and returns a svg string embedding the base64 image.

```example
<<<#let (current-context, value) = ctx.call-function(current-context, "create_svg", (value.image-data-url(read("examples/Typst.svg.png", encoding: none))))
<<<
>>>#let (current-context, value) = ctx.call-function(current-context, "create_svg", (value.image-data-url(data.logo)))
>>>#let logo_svg = bytes(value)
#image(logo_svg)
#str(logo_svg).slice(0, 5)...#str(logo_svg).slice(100, 120)...#str(logo_svg).slice(145, 170)...
```
=== eval-format
Evaluate js directly with formatting data.
```example
#let (current-context, value) = ctx.eval-format(current-context, "`Result for {val1}+{val2} is ${{val1}+{val2}}`", val1: 1, val2: 2)
#value
```
=== transition
A small example to show the difference with transition and without transition.
```example
// new context must be stored
#let (current-context, result) = ctx.call-function(current-context, "changes_data", transition: true)
// With transition:
#result

//changes_data gets called counter will increased and returned. The counter state will be saved with transition.

// Without transition:
#let (current-context, result) = ctx.call-function(current-context, "changes_data")
#result

//changes_data gets called counter will increased and returned. Now the counter will keep the same state as if the function had never been called.

// its mostly recommend always store the returning context (just to be safe)
#let (current-context, result) = ctx.call-function(current-context, "changes_data")

// After:
#result

//Because the counter was not change its still the same as the last call.
```

=== value

Sometimes you want to pass some special data into the javascript.
Like a function callback or a pure json document.

This is possible via the value module.


```example
#ctx.eval-format(current-context, "call_callback({test})", test: value.eval("function() {return `world`;}"))
```

```example
#ctx.eval-format(current-context, "pure_json({test})", test: value.json("{\"arr\":[1,2,3,4]}"))
```

