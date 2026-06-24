# jsesc

[![crates.io](https://img.shields.io/crates/v/jsesc.svg)](https://crates.io/crates/jsesc)
[![docs.rs](https://docs.rs/jsesc/badge.svg)](https://docs.rs/jsesc)
[![CI](https://github.com/trananhtung/jsesc/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/jsesc/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/jsesc.svg)](#license)

**Escape a string for safe embedding in JavaScript or JSON source.**

Handles quotes, backslashes, control characters, line/paragraph separators (`U+2028` /
`U+2029`), and — optionally — every non-ASCII character. A faithful Rust port of the string
escaping of the widely-used [`jsesc`](https://www.npmjs.com/package/jsesc) npm package by
Mathias Bynens.

- **Zero dependencies**, **`#![no_std]`**
- Single / double / backtick quotes, wrapping, JSON mode, ES6 unicode escapes, ASCII-only
  output, minimal escaping, and `<script>`-context safety
- Differential-tested against the reference `jsesc` implementation (60k cases, all options)

## Install

```toml
[dependencies]
jsesc = "0.1"
```

## Usage

```rust
use jsesc::{jsesc, jsesc_with, Options, Quotes};

assert_eq!(jsesc("café"), "caf\\xE9");
assert_eq!(jsesc("foo'bar"), "foo\\'bar");

// JSON-safe (double quotes, wrapped, \uXXXX escapes):
assert_eq!(jsesc_with("hi", &Options::new().json(true)), "\"hi\"");

// ASCII-only output, ES6 astral escapes, backtick quotes:
assert_eq!(jsesc_with("Ā", &Options::new().escape_everything(true)), "\\u0100");
assert_eq!(jsesc_with("\u{1D306}", &Options::new().es6(true)), "\\u{1D306}");
assert_eq!(jsesc_with("`${x}`", &Options::new().quotes(Quotes::Backtick)), "\\`\\${x}\\`");
```

## Scope

This crate ports jsesc's **string escaping**, which is its primary use. The npm package can
also serialize numbers, arrays, and objects; for that, use `serde_json`.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
