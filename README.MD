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

Make sure LLVM is installed locally.

Run unit tests:

> cargo test