# jsesc

[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors-)

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

## Contributors ✨

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome — code, docs, bug reports, ideas, reviews! See the [emoji key](https://allcontributors.org/docs/en/emoji-key) for how each contribution is recognized, and open a PR or issue to get involved.

Thanks goes to these wonderful people:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/trananhtung"><img src="https://avatars.githubusercontent.com/u/30992229?v=4?s=100" width="100px;" alt="Tung Tran"/><br /><sub><b>Tung Tran</b></sub></a><br /><a href="https://github.com/trananhtung/./commits?author=trananhtung" title="Code">💻</a> <a href="#maintenance-trananhtung" title="Maintenance">🚧</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
