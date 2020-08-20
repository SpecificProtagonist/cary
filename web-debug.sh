out_dir=target/web-files

RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown

wasm-bindgen --out-dir $out_dir --web target/wasm32-unknown-unknown/debug/pyrobat.wasm

rm $out_dir/*ts
cp index.html $out_dir/

http $out_dir