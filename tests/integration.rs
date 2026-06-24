//! Integration tests exercising the public API of `jsesc`.

use jsesc::{jsesc, jsesc_with, Options, Quotes};

#[test]
fn embeds_in_single_quoted_literal() {
    assert_eq!(jsesc("Lorem 'ipsum' dolor"), "Lorem \\'ipsum\\' dolor");
    assert_eq!(jsesc("line1\nline2"), "line1\\nline2");
}

#[test]
fn json_mode() {
    let out = jsesc_with("café\t", &Options::new().json(true));
    assert_eq!(out, "\"caf\\u00E9\\t\"");
}

#[test]
fn script_context_safety() {
    let out = jsesc_with("</script><!--", &Options::new().is_script_context(true));
    assert_eq!(out, "<\\/script>\\x3C!--");
}

#[test]
fn ascii_only() {
    let out = jsesc_with("a→b", &Options::new().escape_everything(true).lowercase_hex(true));
    assert_eq!(out, "\\x61\\u2192\\x62");
}

#[test]
fn minimal_keeps_printable_unicode() {
    assert_eq!(jsesc_with("naïve “quote”", &Options::new().minimal(true)), "naïve “quote”");
}
