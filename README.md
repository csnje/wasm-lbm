# wasm-lbm

## About

An implementation of the [Lattice Boltzmann method](https://en.wikipedia.org/wiki/Lattice_Boltzmann_methods) in **Rust** **WebAssembly**.

## Prerequisites

Install [Rust](https://www.rust-lang.org/) and [wasm-pack](https://github.com/rustwasm/wasm-pack).

## Compile

```bash
wasm-pack build --target web
```
or optimised for release
```bash
wasm-pack build --target web --release
```

## Run

Some options to serve the application include:
```bash
# Python 3.x
python3 -m http.server
# Python 2.x
python -m SimpleHTTPServer
# JDK 18 or later
jwebserver
```

Access via a browser at [http://localhost:8000](http://localhost:8000).
