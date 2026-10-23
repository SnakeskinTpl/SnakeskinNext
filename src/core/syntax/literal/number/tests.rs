use super::*;

#[test]
fn fractional_and_exponential_forms_reject_integer_suffixes_at_the_suffix_position() {
    for source in ["1.3u32", "1.3i", "1e3i32", "1E-3u", "1_000.25e+2u64"] {
        let input = ParserInput::with_offset(source, 17);

        let error = number().parse(input).unwrap_err();
        let suffix_start = source.find(['i', 'u']).unwrap();

        assert_eq!(error.location().start_byte, 17);
        assert_eq!(error.location().failure_byte, 17 + suffix_start);
        assert_eq!(error.location().found_char, source[suffix_start..].chars().next());
        assert!(!error.is_recoverable());
    }
}

#[test]
fn incompatible_suffix_errors_propagate_through_literal_and_configuration_parsers() {
    use crate::core::syntax::json::json;
    use crate::core::syntax::literal::literal;

    let error = literal().parse(ParserInput::new(" 1.3u32")).unwrap_err();

    assert_eq!(error.location().failure_byte, 4);
    assert!(!error.is_recoverable());

    let source = "{value: [1e3i32]}";

    let error = json().parse(ParserInput::new(source)).unwrap_err();

    assert_eq!(error.location().failure_byte, source.find("i32").unwrap());
    assert!(!error.is_recoverable());
}

#[test]
fn numbers_preserve_decimal_form_and_exact_spelling_without_host_numeric_limits() {
    for (source, form) in [
        ("0", NumberForm::Integer),
        ("100_333_454", NumberForm::Integer),
        ("9007199254740993", NumberForm::Integer),
        ("999999999999999999999999999999999999999999999999999", NumberForm::Integer),
        ("10.0", NumberForm::Decimal),
        ("100_333.45_67", NumberForm::Decimal),
        ("1e3", NumberForm::Decimal),
        ("1E+3", NumberForm::Decimal),
        ("1.25e-3", NumberForm::Decimal),
        ("1_000.25_50e+1_2", NumberForm::Decimal),
        ("1e999999999999999999999999", NumberForm::Decimal),
    ] {
        let (literal, remaining) = number().parse(ParserInput::new(source)).unwrap();

        assert_eq!(literal.content, source);
        assert_eq!(literal.content.as_ptr(), source.as_ptr());
        assert_eq!(literal.form, form);
        assert_eq!(literal.suffix, None);
        assert!(remaining.is_at_end());
    }
}

#[test]
fn suffixes_preserve_requested_type_and_optional_width() {
    for (source, content, suffix) in [
        ("4i", "4", NumberSuffix::Int { bits: None }),
        ("4i16", "4", NumberSuffix::Int { bits: Some("16") }),
        ("4u", "4", NumberSuffix::UInt { bits: None }),
        ("4u64", "4", NumberSuffix::UInt { bits: Some("64") }),
        ("4f", "4", NumberSuffix::Float { bits: None }),
        ("4f32", "4", NumberSuffix::Float { bits: Some("32") }),
        ("4.25f64", "4.25", NumberSuffix::Float { bits: Some("64") }),
        ("4e3f32", "4e3", NumberSuffix::Float { bits: Some("32") }),
        ("4.25E-2f", "4.25E-2", NumberSuffix::Float { bits: None }),
    ] {
        let (literal, remaining) = number().parse(ParserInput::new(source)).unwrap();

        assert_eq!(literal.content, content);
        assert_eq!(literal.suffix, Some(suffix));
        assert!(remaining.is_at_end());
    }
}

#[test]
fn operators_and_punctuation_remain_for_the_expression_parser() {
    for (source, expected_tail) in [
        ("1..2", "..2"),
        ("1.method()", ".method()"),
        ("1._2", "._2"),
        ("1.", "."),
        ("1.e2", ".e2"),
        ("1..=2", "..=2"),
        ("1+2", "+2"),
        ("1-2", "-2"),
        ("1)", ")"),
        ("1,", ","),
    ] {
        let (literal, remaining) = number().parse(ParserInput::new(source)).unwrap();

        assert_eq!(literal.content, "1");
        assert_eq!(remaining.source_slice(remaining.byte_position()..source.len()), expected_tail);
    }
}

#[test]
fn malformed_numeric_tokens_do_not_succeed_as_valid_prefixes() {
    for source in [
        "1_", "1__2", "1.2_",
        "1foo", "1i32name", "1u_8", "1\u{0436}", "0xFF",
        "1e", "1E+", "1e-", "1e_3", "1e3_", "1e3name",
    ] {
        let error = number().parse(ParserInput::new(source)).unwrap_err();

        assert!(!error.is_recoverable(), "{source:?}");
    }
}

#[test]
fn exponent_sign_is_consumed_but_the_following_operator_is_not() {
    let source = "1.25e-3f64+2";

    let (literal, remaining) = number().parse(ParserInput::new(source)).unwrap();

    assert_eq!(literal.content, "1.25e-3");
    assert_eq!(literal.form, NumberForm::Decimal);
    assert_eq!(literal.suffix, Some(NumberSuffix::Float { bits: Some("64") }));

    assert_eq!(remaining.peek(), Some('+'));
}

#[test]
fn missing_exponent_digits_report_original_position_after_the_sign() {
    let input = ParserInput::with_offset("1e-", 30);

    let error = number().parse(input).unwrap_err();

    assert_eq!(error.location().failure_byte, 33);
    assert_eq!(error.location().found_char, None);
    assert!(!error.is_recoverable());
}

#[test]
fn signs_and_non_decimal_starts_do_not_commit_to_a_number() {
    for source in ["", "-1", "+1", ".5", "_1", "name", "\u{0661}"] {
        let error = number().parse(ParserInput::new(source)).unwrap_err();

        assert!(error.is_recoverable(), "{source:?}");
    }
}

#[test]
fn numeric_literals_keep_original_spans_in_a_bounded_input_slice() {
    use crate::core::syntax::literal::{literal, LiteralKind};

    let source = " \t12_345.67f32 + tail";
    let input = ParserInput::with_offset(source, 20);

    let (value, remaining) = literal().parse(input).unwrap();
    let LiteralKind::Number(number) = value.kind else { panic!("Expected a numeric literal") };

    assert_eq!(value.span, 22..34);
    assert_eq!(remaining.source_slice(value.span), "12_345.67f32");
    assert_eq!(number.content, "12_345.67");
    assert_eq!(remaining.peek(), Some(' '));
}

#[test]
fn missing_digits_after_separator_report_absolute_failure_position() {
    let input = ParserInput::with_offset("12_", 30);

    let error = number().parse(input).unwrap_err();

    assert_eq!(error.location().failure_byte, 33);
    assert_eq!(error.location().found_char, None);
}

#[test]
fn digit_groups_preserve_the_original_spelling_and_stop_before_an_operator() {
    let source = "100_333_454+2";

    let (content, remaining) = digits().parse(ParserInput::new(source)).unwrap();

    assert_eq!(content, "100_333_454");
    assert_eq!(content.as_ptr(), source.as_ptr());
    assert_eq!(remaining.peek(), Some('+'));
}

#[test]
fn underscores_require_digits_on_both_sides() {
    for source in ["1_", "1__2", "1_a", "_1"] {
        assert!(digits().parse(ParserInput::new(source)).is_err(), "{source:?}");
    }
}
