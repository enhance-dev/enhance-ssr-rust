# Enhance SSR Rust Example
This project demonstrates using Enhance to serverside render components in Rust. 

## Download Enhance SSR wasm
Download the latest release of the compiled wasm:
```sh
curl -L https://github.com/enhance-dev/enhance-ssr-wasm/releases/download/v0.0.3/enhance-ssr.wasm.gz | gunzip > wasm/enhance-ssr.wasm
```

## Run
1. Run Server
```sh
cargo run
```
2. load http://localhost:3030/hello/world

## Components and Elements
This example demonstrates a way to read elements from a folder and pass them to the Enhance SSR function as JSON. 
The elements can be formated in several ways:

1. Plain HTML can be used with .html extension. 
2. Pure functions (with a .mjs or .js extenstion) should return HTML and have a function signature as follows:
```javascript
// elements/my-header.mjs
function MyHeader({ html, state }) {
  return html`<style>h1 { color: red; }</style><h1><slot></slot></h1><p>Message: ${state?.store?.message || "no message"}</p>`
}
```


