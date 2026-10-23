use std::borrow::Cow;

use crate::parsec::{input_end, ltrim, seq, ParserInput};

use super::*;

fn complete(source: &str) -> Result<JsonValue<'_>, Box<dyn crate::parsec::ParseError>> {
    let ((value, ()), _) = seq(
        json(),
        ltrim(input_end())
    ).parse(ParserInput::new(source))?;

    Ok(value)
}

#[test]
fn configuration_strings_use_the_same_rules_as_language_literals() {
    for (source, expected) in [
        (r#""\'""#, "'"),
        ("'text'", "text"),
        (r#"r"\n""#, r"\n"),
        ("\"a\tb\"", "a\tb"),
        ("\"\n  foo\n    bar\n\"", "foo\n  bar"),
    ] {
        let value = complete(source).unwrap();

        assert_eq!(value.kind, JsonKind::String(expected.into()), "{source:?}");
        assert_eq!(value.span, 0..source.len());
    }
}

#[test]
fn configuration_object_keys_accept_language_strings_and_keep_source_spans() {
    let source = r#"{'opening': r'\n', r'closing': 'it\'s'}"#;

    let value = complete(source).unwrap();
    let JsonKind::Object(properties) = value.kind else { panic!("Expected an object") };

    assert_eq!(properties[0].key, "opening");
    assert_eq!(&source[properties[0].key_span.clone()], "'opening'");
    assert_eq!(properties[0].value.kind, JsonKind::String(r"\n".into()));
    assert_eq!(properties[1].key, "closing");
    assert_eq!(&source[properties[1].key_span.clone()], "r'closing'");
    assert_eq!(properties[1].value.kind, JsonKind::String("it's".into()));
}

#[test]
fn boolean_values_use_shared_keyword_boundaries() {
    for (source, expected) in [("true,", true), ("false]", false)] {
        let (value, remaining) = json().parse(ParserInput::new(source)).unwrap();

        assert_eq!(value.kind, JsonKind::Bool(expected));
        assert_eq!(value.span, 0..source.len() - 1);

        assert_eq!(remaining.byte_position(), source.len() - 1);
    }

    for source in ["trueValue", "false_", "true1", "false\u{0436}"] {
        assert!(json().parse(ParserInput::new(source)).is_err(), "{source:?}");
    }
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
    use super::super::literal::number;

    for source in ["0", "01", "1e2", "1E-2", "1.25e+2", "1_000_0000", "4i16", "1e2f32", "1e999", "9007199254740993"] {
        let (expected, _) = number().parse(ParserInput::new(source)).unwrap();

        let value = complete(source).unwrap();

        assert_eq!(value.kind, JsonKind::Number(expected));
        assert_eq!(value.span, 0..source.len());
    }
}

#[test]
fn rejects_invalid_numeric_tokens_and_signs_reserved_for_expressions() {
    for source in ["+1", "-2", "-", "1.", "1e", "1e+", "1.e2", "1__2", "1_", "1f32name", "NaN", "Infinity"] {
        assert!(complete(source).is_err(), "Expected invalid number: {source:?}");
    }
}

#[test]
fn nested_configuration_preserves_numeric_spelling_suffix_and_absolute_span() {
    use super::super::literal::{NumberForm, NumberSuffix};

    let source = "{sizes: [9007199254740993u64, 1_000.25e-2f32]}";

    let value = complete(source).unwrap();
    let JsonKind::Object(properties) = value.kind else { panic!("Expected an object") };
    let JsonKind::Array(values) = &properties[0].value.kind else { panic!("Expected an array") };
    let JsonKind::Number(integer) = &values[0].kind else { panic!("Expected an integer") };
    let JsonKind::Number(decimal) = &values[1].kind else { panic!("Expected a decimal") };

    assert_eq!(integer.content, "9007199254740993");
    assert_eq!(integer.form, NumberForm::Integer);
    assert_eq!(integer.suffix, Some(NumberSuffix::UInt { bits: Some("64") }));
    assert_eq!(&source[values[0].span.clone()], "9007199254740993u64");

    assert_eq!(decimal.content, "1_000.25e-2");
    assert_eq!(decimal.form, NumberForm::Decimal);
    assert_eq!(decimal.suffix, Some(NumberSuffix::Float { bits: Some("32") }));
    assert_eq!(&source[values[1].span.clone()], "1_000.25e-2f32");
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
