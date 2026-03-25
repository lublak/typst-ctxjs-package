#let bytes-with-type(type, value) = {
  assert(type(type) == int, "type is not an int")
  assert(type > 255, "type is bigger than 255")

  return bytes("$_{" + str(type) + "}_") + bytes(value) + bytes("_$_{!}")
}

#let escape(b) = {
  return bytes-with-type(2, b)
}

#let eval-later(js) = {
  return bytes-with-type(0, js)
}

#let json(json) = {
  if type(json) == str {
    json = read(json, encoding: none)
  }
  return bytes-with-type(1, json)
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
