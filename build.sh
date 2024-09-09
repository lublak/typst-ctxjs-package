rustup target add wasm32-wasi
cp README.md typst-package/
cp LICENSE typst-package/
cargo build --release --target wasm32-wasi
wasi-stub -r 0 ./target/wasm32-wasi/release/ctxjs.wasm -o typst-package/ctxjs.wasm
wasm-opt typst-package/ctxjs.wasm -O3 --enable-bulk-memory -o typst-package/ctxjs.wasm