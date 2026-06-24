//! # jsesc — escape a string for JavaScript or JSON
//!
//! Escape a string so it can be safely embedded in JavaScript or JSON source — handling
//! quotes, backslashes, control characters, line/paragraph separators, and (optionally)
//! every non-ASCII character. A faithful Rust port of the string-escaping of the widely-used
//! [`jsesc`](https://www.npmjs.com/package/jsesc) npm package by Mathias Bynens.
//!
//! ```
//! use jsesc::jsesc;
//!
//! assert_eq!(jsesc("café"), "caf\\xE9");
//! assert_eq!(jsesc("foo'bar"), "foo\\'bar");
//! ```
//!
//! Use [`jsesc_with`] / [`Options`] for double/backtick quotes, wrapping, JSON mode, ES6
//! unicode escapes, ASCII-only output, and more:
//!
//! ```
//! use jsesc::{jsesc_with, Options, Quotes};
//!
//! assert_eq!(jsesc_with("hi", &Options::new().json(true)), "\"hi\"");
//! assert_eq!(jsesc_with("ab", &Options::new().escape_everything(true)), "\\x61\\x62");
//! ```
//!
//! **Zero dependencies** and `#![no_std]`.

#![no_std]
#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/jsesc/0.1.0")]
#![allow(clippy::struct_excessive_bools, clippy::format_push_string)]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

// Compile-test the README's examples as part of `cargo test`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

/// The quote style to escape for (and wrap with).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quotes {
    /// Single quotes (`'`).
    Single,
    /// Double quotes (`"`).
    Double,
    /// Backticks (`` ` ``).
    Backtick,
}

/// Options for [`jsesc_with`].
#[derive(Debug, Clone, Default)]
pub struct Options {
    quotes: Option<Quotes>,
    wrap: Option<bool>,
    es6: bool,
    escape_everything: bool,
    minimal: bool,
    is_script_context: bool,
    json: bool,
    lowercase_hex: bool,
}

impl Options {
    /// Default options (single quotes, no wrapping, escape non-ASCII).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Which quote character to escape (and wrap with). Defaults to [`Quotes::Single`]
    /// (or [`Quotes::Double`] when `json` is set).
    #[must_use]
    pub fn quotes(mut self, quotes: Quotes) -> Self {
        self.quotes = Some(quotes);
        self
    }

    /// Wrap the output in the quote character. Defaults to `false` (or `true` when `json`).
    #[must_use]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Use ES6 unicode code point escapes (`\u{1D306}`) for astral symbols.
    #[must_use]
    pub fn es6(mut self, es6: bool) -> Self {
        self.es6 = es6;
        self
    }

    /// Escape every character, including printable ASCII.
    #[must_use]
    pub fn escape_everything(mut self, value: bool) -> Self {
        self.escape_everything = value;
        self
    }

    /// Only escape the strictly necessary characters (quotes, backslash, line breaks, and
    /// invisible whitespace), leaving printable non-ASCII as-is.
    #[must_use]
    pub fn minimal(mut self, value: bool) -> Self {
        self.minimal = value;
        self
    }

    /// Escape `</script`, `</style`, and `<!--` so the output is safe inside a `<script>`
    /// or `<style>` element.
    #[must_use]
    pub fn is_script_context(mut self, value: bool) -> Self {
        self.is_script_context = value;
        self
    }

    /// Produce JSON-compatible output (double quotes, wrapped, `\uXXXX` escapes only).
    #[must_use]
    pub fn json(mut self, value: bool) -> Self {
        self.json = value;
        self
    }

    /// Emit hexadecimal digits in lowercase.
    #[must_use]
    pub fn lowercase_hex(mut self, value: bool) -> Self {
        self.lowercase_hex = value;
        self
    }

    fn resolved_quotes(&self) -> Quotes {
        self.quotes.unwrap_or(if self.json {
            Quotes::Double
        } else {
            Quotes::Single
        })
    }

    fn resolved_wrap(&self) -> bool {
        self.wrap.unwrap_or(self.json)
    }
}

/// Escape `string` for embedding in JavaScript source, using the default options.
///
/// ```
/// # use jsesc::jsesc;
/// assert_eq!(jsesc("a\tb"), "a\\tb");
/// ```
#[must_use]
pub fn jsesc(string: &str) -> String {
    jsesc_with(string, &Options::new())
}

/// Escape `string` with the given [`Options`].
#[must_use]
pub fn jsesc_with(string: &str, options: &Options) -> String {
    let quote = match options.resolved_quotes() {
        Quotes::Single => '\'',
        Quotes::Double => '"',
        Quotes::Backtick => '`',
    };

    let units: Vec<u16> = string.encode_utf16().collect();
    let mut result = String::with_capacity(string.len());

    let mut index = 0;
    while index < units.len() {
        let unit = units[index];

        if is_high_surrogate(unit) && index + 1 < units.len() && is_low_surrogate(units[index + 1])
        {
            let low = units[index + 1];
            let code_point =
                (u32::from(unit) - 0xD800) * 0x400 + (u32::from(low) - 0xDC00) + 0x1_0000;
            if options.minimal {
                push_scalar(&mut result, code_point);
            } else if options.es6 {
                result.push_str(&format!(
                    "\\u{{{}}}",
                    hex(code_point, options.lowercase_hex)
                ));
            } else {
                result.push_str(&four_hex_escape(unit, options.lowercase_hex));
                result.push_str(&four_hex_escape(low, options.lowercase_hex));
            }
            index += 2;
            continue;
        }

        if is_surrogate(unit) {
            // Lone surrogate.
            result.push_str(&four_hex_escape(unit, options.lowercase_hex));
            index += 1;
            continue;
        }

        let escape = options.escape_everything || is_quote(unit) || !is_safe(unit);
        if escape {
            let next = units.get(index + 1).copied();
            result.push_str(&escape_unit(unit, next, options, quote));
        } else {
            push_scalar(&mut result, u32::from(unit));
        }
        index += 1;
    }

    if quote == '`' {
        result = result.replace("${", "\\${");
    }
    if options.is_script_context {
        result = escape_script_tags(&result);
        let comment = if options.json {
            "\\u003C!--"
        } else {
            "\\x3C!--"
        };
        result = result.replace("<!--", comment);
    }
    if options.resolved_wrap() {
        let mut wrapped = String::with_capacity(result.len() + 2);
        wrapped.push(quote);
        wrapped.push_str(&result);
        wrapped.push(quote);
        result = wrapped;
    }
    result
}

fn is_high_surrogate(unit: u16) -> bool {
    (0xD800..=0xDBFF).contains(&unit)
}

fn is_low_surrogate(unit: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&unit)
}

fn is_surrogate(unit: u16) -> bool {
    (0xD800..=0xDFFF).contains(&unit)
}

fn is_quote(unit: u16) -> bool {
    matches!(unit, 0x22 | 0x27 | 0x60)
}

/// The characters jsesc leaves untouched in the default (non-`escapeEverything`) mode:
/// `[ !#-&(-[]-_a-~]`.
fn is_safe(unit: u16) -> bool {
    matches!(unit, 0x20 | 0x21 | 0x23..=0x26 | 0x28..=0x5B | 0x5D..=0x5F | 0x61..=0x7E)
}

/// Whitespace that `minimal` mode still escapes (`regexWhitespace` in the reference).
fn is_special_whitespace(unit: u16) -> bool {
    matches!(
        unit,
        0xA0 | 0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000
    )
}

fn hex(code: u32, lowercase: bool) -> String {
    if lowercase {
        format!("{code:x}")
    } else {
        format!("{code:X}")
    }
}

fn four_hex_escape(unit: u16, lowercase: bool) -> String {
    format!("\\u{:0>4}", hex(u32::from(unit), lowercase))
}

/// Push the scalar value (a non-surrogate code point) as a `char`.
fn push_scalar(result: &mut String, code: u32) {
    if let Some(c) = char::from_u32(code) {
        result.push(c);
    }
}

fn escape_unit(unit: u16, next: Option<u16>, options: &Options, quote: char) -> String {
    // `\0` is escaped as `\0` unless it is followed by a digit (which would form `\01`), or
    // in JSON mode.
    if unit == 0 && !options.json && !matches!(next, Some(0x30..=0x39)) {
        return "\\0".to_string();
    }

    if is_quote(unit) {
        let character = char::from_u32(u32::from(unit)).unwrap_or('"');
        if character == quote || options.escape_everything {
            return format!("\\{character}");
        }
        return character.to_string();
    }

    if let Some(single) = single_escape(unit) {
        return single.to_string();
    }

    if options.minimal && !is_special_whitespace(unit) {
        let mut out = String::new();
        push_scalar(&mut out, u32::from(unit));
        return out;
    }

    let hex = hex(u32::from(unit), options.lowercase_hex);
    if options.json || hex.len() > 2 {
        return format!("\\u{hex:0>4}");
    }
    format!("\\x{hex:0>2}")
}

fn single_escape(unit: u16) -> Option<&'static str> {
    match unit {
        0x5C => Some("\\\\"),
        0x08 => Some("\\b"),
        0x0C => Some("\\f"),
        0x0A => Some("\\n"),
        0x0D => Some("\\r"),
        0x09 => Some("\\t"),
        _ => None,
    }
}

/// Insert a backslash before the `/` of `</script` / `</style` (case-insensitive), matching
/// the reference's `/<\/(script|style)/gi` → `<\/$1`.
fn escape_script_tags(string: &str) -> String {
    let mut out = String::with_capacity(string.len());
    let mut index = 0;
    while index < string.len() {
        let rest = &string[index..];
        if let Some(after) = rest.strip_prefix("</") {
            if starts_with_ignore_ascii_case(after, "script")
                || starts_with_ignore_ascii_case(after, "style")
            {
                out.push_str("<\\/");
                index += 2;
                continue;
            }
        }
        let character = rest.chars().next().expect("non-empty remainder");
        out.push(character);
        index += character.len_utf8();
    }
    out
}

fn starts_with_ignore_ascii_case(haystack: &str, prefix: &str) -> bool {
    haystack.len() >= prefix.len()
        && haystack.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        assert_eq!(jsesc("café"), "caf\\xE9");
        assert_eq!(jsesc("foo'bar"), "foo\\'bar");
        assert_eq!(jsesc("\t\n\\"), "\\t\\n\\\\");
        assert_eq!(jsesc("plain ASCII!"), "plain ASCII!");
    }

    #[test]
    fn quotes_and_wrap() {
        assert_eq!(
            jsesc_with("a\"b", &Options::new().quotes(Quotes::Double)),
            "a\\\"b"
        );
        assert_eq!(
            jsesc_with("x`${y}", &Options::new().quotes(Quotes::Backtick)),
            "x\\`\\${y}"
        );
        assert_eq!(jsesc_with("hi", &Options::new().wrap(true)), "'hi'");
        assert_eq!(jsesc_with("hi", &Options::new().json(true)), "\"hi\"");
    }

    #[test]
    fn astral_and_es6() {
        assert_eq!(jsesc("\u{1D306}"), "\\uD834\\uDF06");
        assert_eq!(
            jsesc_with("\u{1D306}", &Options::new().es6(true)),
            "\\u{1D306}"
        );
        assert_eq!(
            jsesc_with("\u{1D306}", &Options::new().minimal(true)),
            "\u{1D306}"
        );
    }

    #[test]
    fn nulls() {
        assert_eq!(jsesc("\0"), "\\0");
        assert_eq!(jsesc("\0a"), "\\0a");
        assert_eq!(jsesc("\u{0}1"), "\\x001"); // null followed by a digit
    }

    #[test]
    fn escape_everything_and_minimal() {
        assert_eq!(
            jsesc_with("ab", &Options::new().escape_everything(true)),
            "\\x61\\x62"
        );
        assert_eq!(jsesc_with("café", &Options::new().minimal(true)), "café");
        assert_eq!(jsesc_with("a b", &Options::new().minimal(true)), "a b");
        assert_eq!(
            jsesc_with("café", &Options::new().lowercase_hex(true)),
            "caf\\xe9"
        );
    }

    #[test]
    fn script_context() {
        assert_eq!(
            jsesc_with("</script>", &Options::new().is_script_context(true)),
            "<\\/script>"
        );
        assert_eq!(
            jsesc_with("<!--x", &Options::new().is_script_context(true)),
            "\\x3C!--x"
        );
    }

    #[test]
    fn line_separators_and_control() {
        // U+2028 / U+2029 break JS string literals and must be escaped.
        assert_eq!(jsesc("a\u{2028}b"), "a\\u2028b");
        assert_eq!(jsesc("a\u{2029}b"), "a\\u2029b");
        // A C1 control / high BMP code point uses a four-digit escape.
        assert_eq!(jsesc("\u{0100}"), "\\u0100");
    }
}
