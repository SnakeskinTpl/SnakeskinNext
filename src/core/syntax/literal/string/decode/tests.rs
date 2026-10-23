use super::*;

#[test]
fn plain_unicode_and_empty_content_remain_borrowed() {
    for source in ["", "hello", "\u{0436}\u{1f600}"] {
        let value = decode_string(source, 1).unwrap();

        assert_eq!(value, source);
        assert!(matches!(value, Cow::Borrowed(_)));
        assert_eq!(value.as_ptr(), source.as_ptr());
    }
}

#[test]
fn supported_escapes_decode_to_owned_content() {
    let source = r#"a\"\\\/\b\f\n\r\t"#;
    let expected = "a\"\\/\u{8}\u{c}\n\r\t";

    let value = decode_string(source, 7).unwrap();

    assert_eq!(value, expected);
    assert!(matches!(value, Cow::Owned(_)));
}

#[test]
fn single_quote_escape_and_literal_tab_follow_snakeskin_rules() {
    let escaped = decode_string(r"it\'s", 10).unwrap();
    let tabbed = decode_string("a\tb", 10).unwrap();

    assert_eq!(escaped, "it's");
    assert_eq!(tabbed, "a\tb");
    assert!(matches!(tabbed, Cow::Borrowed(_)));
}

#[test]
fn other_unescaped_control_characters_are_rejected() {
    for source in ["a\nb", "a\rb", "a\0b", "a\u{1f}b"] {
        let error = decode_string(source, 20).unwrap_err();

        assert_eq!(error.location().failure_byte, 21);
        assert!(!error.is_recoverable());
    }
}

#[test]
fn unicode_escapes_decode_bmp_characters_and_surrogate_pairs() {
    for (source, expected) in [
        (r"\u0041", "A"),
        (r"\u0000", "\0"),
        (r"\uD7FF", "\u{d7ff}"),
        (r"\uE000", "\u{e000}"),
        (r"\uD83D\uDE00", "\u{1f600}"),
        (r"\uDBFF\uDFFF", "\u{10ffff}"),
    ] {
        let value = decode_string(source, 1).unwrap();

        assert_eq!(value, expected, "{source:?}");
    }
}

#[test]
fn malformed_escapes_report_the_original_backslash_after_multibyte_text() {
    for suffix in [
        "\\", r"\q", r"\u12", r"\uZZZZ", r"\uDC00",
        r"\uD800", r"\uD800x", r"\uD800\xDC00", r"\uD800\u0041", r"\uD800\u12",
    ] {
        let source = format!("\u{0436}{suffix}");

        let error = decode_string(&source, 30).unwrap_err();

        assert_eq!(error.location().failure_byte, 32, "{source:?}");
        assert_eq!(error.location().found_char, Some('\\'));
        assert!(!error.is_recoverable());
    }
}

#[test]
fn hexadecimal_quad_reads_four_digits_and_leaves_the_suffix() {
    let mut chars = "00aF!".char_indices();

    let value = hex_quad(&mut chars);

    assert_eq!(value, Some(0xaf));
    assert_eq!(chars.next(), Some((4, '!')));

    for source in ["", "123", "0x12", "gggg"] {
        assert_eq!(hex_quad(&mut source.char_indices()), None, "{source:?}");
    }
}
