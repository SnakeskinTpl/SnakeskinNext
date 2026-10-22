use std::borrow::Cow;

use crate::parsec::{ltrim, not, seq, take, ParserInput};

use super::*;

fn complete(source: &str) -> Result<JsonValue<'_>, Box<dyn crate::parsec::ParseError>> {
    let ((value, ()), _) = seq(
        json(),
        ltrim(not(take(|_, _, _| true, 1..=1)))
    ).parse(ParserInput::new(source))?;

    Ok(value)
}

#[test]
fn parses_nested_configuration_with_identifier_and_quoted_keys() {
    let source = r#"{ opening: "<", "closing": ">", options: [true, false, null, {depth: 2}] }"#;

    let value = complete(source).unwrap();

    assert_eq!(value.span, 0..source.len());

    let JsonKind::Object(properties) = value.kind else { panic!("Expected an object") };

    assert_eq!(properties.len(), 3);
    assert_eq!(properties[0].key, "opening");
    assert_eq!(properties[0].value.kind, JsonKind::String(Cow::Borrowed("<")));
    assert_eq!(&source[properties[1].key_span.clone()], "\"closing\"");

    let JsonKind::Array(elements) = &properties[2].value.kind else { panic!("Expected an array") };

    assert_eq!(elements.len(), 4);
    assert_eq!(elements[0].kind, JsonKind::Bool(true));
    assert_eq!(elements[1].kind, JsonKind::Bool(false));
    assert_eq!(elements[2].kind, JsonKind::Null);
    assert!(matches!(elements[3].kind, JsonKind::Object(_)));
}

#[test]
fn preserves_duplicate_properties_in_source_order() {
    let source = r#"{opening: "<", opening: "["}"#;

    let value = complete(source).unwrap();
    let JsonKind::Object(properties) = value.kind else { panic!("Expected an object") };

    assert_eq!(properties.len(), 2);
    assert_eq!(properties[0].key, properties[1].key);
    assert_ne!(properties[0].key_span, properties[1].key_span);
    assert_eq!(properties[1].value.kind, JsonKind::String(Cow::Borrowed("[")));
}

#[test]
fn empty_containers_and_multiline_whitespace_are_supported() {
    assert_eq!(complete(" [ \r\n ] ").unwrap().kind, JsonKind::Array(vec![]));
    assert_eq!(complete(" { \n } ").unwrap().kind, JsonKind::Object(vec![]));
}

#[test]
fn plain_strings_borrow_source_and_escapes_are_decoded() {
    let source = String::from("\"я🦀\"");

    let value = complete(&source).unwrap();
    let JsonKind::String(Cow::Borrowed(text)) = value.kind else { panic!("Expected borrowed string") };

    assert_eq!(text, "я🦀");
    assert_eq!(text.as_ptr(), source[1..].as_ptr());

    let value = complete(r#""\"\\\/\b\f\n\r\t\u044F\uD83E\uDD80""#).unwrap();

    assert_eq!(value.kind, JsonKind::String(Cow::Owned("\"\\/\u{8}\u{c}\n\r\tя🦀".into())));
}

#[test]
fn malformed_strings_report_fatal_errors() {
    for source in [r#""unterminated"#, r#""\q""#, r#""\u12xz""#, r#""\uD800""#,
        r#""\uDC00""#, r#""\uD800\u0041""#, "\"raw\nnewline\""] {
        let error = complete(source).unwrap_err();

        assert!(!error.is_recoverable(), "Expected committed string error: {source:?}");
        assert!(error.byte_position() <= source.len());
    }
}

#[test]
fn parses_integer_fraction_and_exponent_forms() {
    for (source, expected) in [("0", 0.0), ("-2", -2.0), ("1e2", 100.0),
        ("1E-2", 0.01), ("-1.25e+2", -125.0)] {
        let value = complete(source).unwrap();

        assert_eq!(value.kind, JsonKind::Number(expected));
    }
}

#[test]
fn rejects_invalid_numbers_and_overflow() {
    for source in ["+1", "01", "-", "1.", "1e", "1e+", "1.e2", "1e999", "NaN", "Infinity"] {
        assert!(complete(source).is_err(), "Expected invalid number: {source:?}");
    }
}

#[test]
fn malformed_containers_are_not_accepted_as_empty_or_partial_values() {
    for source in ["[", "{", "[1,]", "{a:1,}", "[1 2]", "{a 1}", "{a:}",
        "{a:[1,]}", "[true", "{a:1", "[,1]", "{1:2}"] {
        let error = complete(source).unwrap_err();

        assert!(!error.is_recoverable(), "Expected committed container error: {source:?}");
    }
}

#[test]
fn embedded_values_preserve_original_positions_and_remaining_input() {
    let source = "! \t{a: [2]} tail";
    let mut input = ParserInput::new(source);
    input.set_byte_position(1);

    let (value, remaining) = json().parse(input).unwrap();

    assert_eq!(value.span, 3..11);
    assert_eq!(remaining.byte_position(), 11);
    assert_eq!(remaining.peek(), Some(' '));

    let JsonKind::Object(properties) = value.kind else { panic!("Expected an object") };

    assert_eq!(properties[0].key_span, 4..5);
    assert_eq!(properties[0].value.span, 7..10);
}
