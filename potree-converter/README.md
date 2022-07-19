<div align="center">

  <h1><code>potree-converter</code></h1>

<strong>A NPM package for client side conversion of point clouds into <a href="https://github.com/potree/potree"> potree format </a>.</strong>


</div>

## About

Exposes potree conversion via `wasm.process_array_buffer("pcd", new Uint8Array(buf))`; 

Currently works using webpack. Not compatible with Parcel due to [sync imports not working](https://github.com/parcel-bundler/parcel/issues/647) 


## 🚴 Usage

### 🛠️ Build with `wasm-pack build`

```
wasm-pack build
```
