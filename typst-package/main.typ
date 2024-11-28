#import "lib.typ" as ctxjs

#{
  _ = ctxjs.create-context("test")
  let test = ctxjs.eval-format("test", "((callback) => {{ return callback(); }})({test})", ("test":ctxjs.eval-later("test", "() => {return 5;}"),))
}