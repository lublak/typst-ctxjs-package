function create_svg(data) {
    return `<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <image width="100\" height="100" xlink:href="` + data + `"/>
</svg>`
}