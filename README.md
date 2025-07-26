# wasm-lbm

> **WARNING!**
> 
> This project is impacted by the sunsetting of the [rustwasm](https://github.com/rustwasm) Github Organisation as described [here](https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org/).

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
