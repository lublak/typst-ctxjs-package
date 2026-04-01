#import "internal.typ" as _internal

#let escape(b) = {
  return _internal.bytes-with-type(2, b)
}

#let eval-later(js) = {
  return _internal.bytes-with-type(0, js)
}

#let json(json) = {
  if type(json) == str {
    json = read(json, encoding: none)
  }
  return _internal.bytes-with-type(1, json)
}


#let image-data-url(data, format: auto) = {
  if type(data) == str {
    data = read(data, encoding: none)
  }
  if format == auto {
    format = ""
  }
  return str(_internal.wasm.image_data_url(data))
}
