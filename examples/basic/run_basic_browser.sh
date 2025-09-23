RUSTC_BOOTSTRAP=1 cargo install --git https://github.com/thecoshman/http

cargo install wasm-pack
wasm-pack build --target web
http


