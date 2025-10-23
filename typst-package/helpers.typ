#let eval-later(js, type-field: "$type") = {
  let o = (value: js)
  o.insert(type-field, "eval")
  return o
}

#let json(json, type-field: "$type") = {
  if type(json) == str {
    json = read(json, encoding: none)
  }
  let o = (value: json)
  o.insert(type-field, "json")
  return o
}

#let string-to-bytes(data) = {
  let data = data
  if type(data) == str {
    data = bytes(data)
  } else if type(data) == array {
    data = bytes(data)
  } else if type(data) == content {
    data = bytes(data.text)
  }
  data
}

#let image-data-url(data, format: auto) = {
  if type(data) == str {
    data = read(data, encoding: none)
  }
  if format == auto {
    format = ""
  }
  return str(ctxjs.image_data_url(data))
}
