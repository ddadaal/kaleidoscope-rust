# Kaleidoscope Rust Version

!! WIP !!

Building LLVM's official [Kaleidoscope](https://llvm.org/docs/tutorial/index.html) tutorial language in Rust.

## Features

- [inkwell](https://crates.io/crates/inkwell) for safe Rust LLVM binding
- [lexer](core/src/lexer), [parser](core/src/parser) with thorough unit tests
- Complete error report mechanism utilizing `?` operator
- Extensive use of macros to reduce boilerplate code
- [lexer implemented as `Iterator<Item=Result<Token, LexerError>>`](core/src/lexer/lexer.rs) for better abstraction

## Dev

Currently it uses LLVM 10.

[llvmenv](https://github.com/termoshtt/llvmenv) is used to manage llvm builds.

The quickest to get started in Linux is do the following. You may consult llvmenv's documentations to config llvm environments.

```bash
# 1. Install llvm from your distro's package manager
#    Arch's pacman is used here as an example
sudo pacman -S llvm

# 2. Install llvmenv through cargo
cargo install llvmenv

# 3. Set the system executable as the llvm to be used
llvmenv global system
```

Run unit tests:

> cargo test