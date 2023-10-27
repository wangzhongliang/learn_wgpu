# PC运行
```
cargo run
```

# web运行(WebGPU)
```
RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build -t web
```

# web运行(WebGL2)
修改Cargo.toml中`[target.'cfg(target_arch = "wasm32")'.dependencies]`下面的`wgpu = { version="0.17" }`为`wgpu = { version="0.17", features= ["webgl"]}`

再执行
```
wasm-pack build -t web
```