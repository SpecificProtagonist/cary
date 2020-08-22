out_dir=target/web-files

RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --release --target wasm32-unknown-unknown

wasm-bindgen --out-dir $out_dir --web target/wasm32-unknown-unknown/release/cary.wasm

#TODO: optimise wasm (binaryen)

rm $out_dir/*ts
cp index.html $out_dir/

#TODO: zip it all up