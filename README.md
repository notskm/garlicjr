# 🕹 GarlicJr: A Game Boy emulator on the web
[![CI](https://github.com/notskm/garlicjr/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/notskm/garlicjr/actions/workflows/ci.yml)  [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](./COPYING)

[Run the emulator in your browser!](https://notskm.github.io/garlicjr)

## Building

### Native
```sh
cargo build --release
```

### WebAssembly
The front-end for GarlicJr is built using [Trunk](https://trunkrs.dev/) when compiling for WebAssembly. Please follow their getting started guide before continuing.

```sh
cd garlicjr_dbg
trunk build --release
```
