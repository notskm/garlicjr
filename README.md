# garlicjr

[![Build Status](https://github.com/notskm/garlicjr/workflows/CI/badge.svg)](https://github.com/notskm/garlicjr/actions?query=workflow%3ACI)
[![Clang-Format Status](https://github.com/notskm/garlicjr/workflows/clang-format/badge.svg)](https://github.com/notskm/garlicjr/actions?query=workflow%3Aclang-format)
[![CMake-Format Status](https://github.com/notskm/garlicjr/workflows/cmake-format/badge.svg)](https://github.com/notskm/garlicjr/actions?query=workflow%3Acmake-format)
[![License](https://img.shields.io/github/license/notskm/garlicjr)](LICENSE.txt)

## Building

### Prerequisites

* C++17 compliant compiler
* [CMake](https://cmake.org/download/)
* [Conan](https://conan.io/downloads.html)

### Setting up

```sh
git clone https://github.com/notskm/garlicjr
cd garlicjr
```

```sh
mkdir build
cd build
```

### Makefile generators (Make, Ninja, etc.)

```sh
cmake .. -GNinja -DCMAKE_BUILD_TYPE=Release
cmake --build .
```

### Multi generators (Visual Studio, etc.)

```sh
cmake .. -G "Visual Studio 16 2019"
cmake --build . --config Release
```

### CMake options

|       Option       | Default | Description                        |
| :----------------: | :-----: | ---------------------------------- |
|     RUN_CONAN      |   ON    | Runs `conan install` automatically |
|    BUILD_TESTS     |   OFF   | Builds the tests                   |
| WARNINGS_AS_ERRORS |   OFF   | Treat compiler warnings as errors  |

## Authors

* [@notskm](https://github.com/notskm)

See also the list of [contributors](https://github.com/notskm/garlicjr) who participated in this project.
