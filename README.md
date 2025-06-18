# 🕹 GarlicJr: A Game Boy emulator on the web
[![CI](https://github.com/notskm/garlicjr/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/notskm/garlicjr/actions/workflows/ci.yml)  [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](./COPYING)

🚀 [Run the emulator in your browser!](https://notskm.github.io/garlicjr)

GarlicJr is aiming to be a cross-platform, [cycle accurate](https://retrocomputing.stackexchange.com/a/1195) Game Boy emulator.

## Key features
- Cycle accurate emulation
- Compiles to WebAssembly
- Linux & Windows support
  - MacOS is untested
- Debugging features
- Low to No-dependency core
  - Should compile virtually anywhere

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

## License
GarlicJr is licensed under the [GNU General Public License v3](https://www.gnu.org/licenses/gpl-3.0.en.html). See the [COPYING](./COPYING) file for details.
