use crate::core::syntax::expression::{expression, BinaryOperator, Expression, UnaryOperator};
use crate::parsec::{input_end, ltrim, seq, Parser, ParserInput};

use super::{infer, Type, TypeEnvironment, TypeErrorKind};

fn parse(source: &str) -> Expression<'_> {
    let ((parsed, ()), _) = seq(expression(), ltrim(input_end()))
        .parse(ParserInput::new(source))
        .unwrap();

    parsed
}

#[test]
fn infers_boolean_expressions_in_an_empty_environment() {
    let environment = TypeEnvironment::new();

    for source in ["true", "false", "!(true)", "!!false", "true && !false", "false || true", "true == false", "true != false", "<Bool>!false"] {
        let parsed = parse(source);

        let inferred = infer(&parsed, &environment).unwrap();

        assert_eq!(inferred.ty, Type::Bool, "{source:?}");
    }
}

#[test]
fn resolves_names_from_the_environment_without_modifying_it_or_the_ast() {
    let environment = TypeEnvironment::from([("enabled", Type::Bool), ("visible", Type::Bool)]);
    let parsed = parse("!enabled || visible");
    let original = parsed.clone();

    let inferred = infer(&parsed, &environment).unwrap();

    assert_eq!(inferred.ty, Type::Bool);
    assert_eq!(parsed, original);
    assert_eq!(environment.len(), 2);
    assert_eq!(environment["enabled"], Type::Bool);
}

#[test]
fn unknown_identifiers_keep_their_absolute_source_range() {
    let (parsed, _) = expression().parse(ParserInput::with_offset(" !missing", 10)).unwrap();

    let error = infer(&parsed, &TypeEnvironment::new()).unwrap_err();

    assert_eq!(error.span, 12..19);
    assert_eq!(error.kind, TypeErrorKind::UnknownIdentifier { name: "missing".into() });
}

#[test]
fn boolean_values_do_not_support_arithmetic_or_ordering() {
    for (source, operator) in [("true + false", BinaryOperator::Add), ("true < false", BinaryOperator::Less)] {
        let parsed = parse(source);

        let error = infer(&parsed, &TypeEnvironment::new()).unwrap_err();

        assert_eq!(error.span, 5..6);
        assert_eq!(error.kind, TypeErrorKind::InvalidBinary {
            operator, left: Type::Bool, right: Type::Bool,
        });
    }

    let parsed = parse("-true");

    let error = infer(&parsed, &TypeEnvironment::new()).unwrap_err();

    assert_eq!(error.span, 0..1);
    assert_eq!(error.kind, TypeErrorKind::InvalidUnary {
        operator: UnaryOperator::Negate, operand: Type::Bool,
    });
}

#[test]
fn casts_resolve_bool_and_report_unknown_type_names() {
    let environment = TypeEnvironment::from([("flag", Type::Bool)]);

    assert_eq!(infer(&parse("<Bool>flag"), &environment).unwrap().ty, Type::Bool);

    let error = infer(&parse("<Missing>flag"), &environment).unwrap_err();

    assert_eq!(error.span, 1..8);
    assert_eq!(error.kind, TypeErrorKind::UnknownType { name: "Missing".into() });

    let error = infer(&parse("<Bool>missing"), &environment).unwrap_err();

    assert_eq!(error.span, 6..13);
    assert!(matches!(error.kind, TypeErrorKind::UnknownIdentifier { .. }));
}

#[test]
fn unsupported_constructs_are_explicit_errors_instead_of_unknown_types() {
    for source in ["null", "call()", "object.field", "items[index]", "<Opt<Bool>>true"] {
        let parsed = parse(source);

        let error = infer(&parsed, &TypeEnvironment::new()).unwrap_err();

        assert_eq!(error.span, if source == "<Opt<Bool>>true" { 1..10 } else { 0..source.len() }, "{source:?}");
        assert!(matches!(error.kind, TypeErrorKind::Unsupported { .. }), "{source:?}");
    }
}

#[test]
fn numeric_literals_use_suffixes_and_defaults_without_losing_exact_spelling() {
    for (source, expected) in [
        ("1", Type::Int { bits: 32 }),
        ("1.5", Type::Float { bits: 64 }),
        ("1e3", Type::Float { bits: 64 }),
        ("1i64", Type::Int { bits: 64 }),
        ("1u", Type::UInt { bits: 32 }),
        ("1f32", Type::Float { bits: 32 }),
        ("1_000.50e-3f64", Type::Float { bits: 64 }),
    ] {
        let parsed = parse(source);

        let inferred = infer(&parsed, &TypeEnvironment::new()).unwrap();

        assert_eq!(inferred.ty, expected, "{source:?}");
        let Some(crate::core::syntax::literal::LiteralKind::Number(number)) = inferred.literal else { panic!("Expected literal data") };

        assert!(source.starts_with(number.content));
    }
}

#[test]
fn literal_data_is_preserved_but_computed_expressions_are_not_marked_as_literals() {
    use crate::core::syntax::literal::LiteralKind;

    let environment = TypeEnvironment::from([("flag", Type::Bool)]);

    for source in ["true", "(true)", "<Bool>true"] {
        let inferred = infer(&parse(source), &environment).unwrap();

        assert_eq!(inferred.literal, Some(LiteralKind::Bool(true)));
    }

    for source in ["flag", "!true", "true && false", "1 + 2", "-1"] {
        let inferred = infer(&parse(source), &environment).unwrap();

        assert_eq!(inferred.literal, None, "{source:?}");
    }
}

#[test]
fn string_literals_preserve_decoded_values_for_inlining() {
    use crate::core::syntax::literal::LiteralKind;

    let parsed = parse(r#""hello\nworld""#);

    let inferred = infer(&parsed, &TypeEnvironment::new()).unwrap();

    assert_eq!(inferred.ty, Type::Str);
    assert_eq!(inferred.literal, Some(LiteralKind::String("hello\nworld".into())));

    let inferred = infer(&parse(r#"<Str>"text""#), &TypeEnvironment::new()).unwrap();

    assert_eq!(inferred.ty, Type::Str);
    assert_eq!(inferred.literal, Some(LiteralKind::String("text".into())));
}

#[test]
fn string_variables_and_literals_share_a_type_but_not_literal_data() {
    let environment = TypeEnvironment::from([("name", Type::Str)]);

    let literal = infer(&parse("\"hello\""), &environment).unwrap();
    let variable = infer(&parse("name"), &environment).unwrap();
    let comparison = infer(&parse("name == \"hello\""), &environment).unwrap();

    assert_eq!(literal.ty, Type::Str);
    assert_eq!(variable.ty, literal.ty);
    assert!(literal.literal.is_some());
    assert!(variable.literal.is_none());
    assert_eq!(comparison.ty, Type::Bool);
}

#[test]
fn explicit_operand_types_supply_context_in_either_order_before_defaults() {
    let environment = TypeEnvironment::from([("wide", Type::Int { bits: 64 })]);

    for source in ["wide + 1", "1 + wide", "1 + (2 + wide)", "1 + 2i64", "<Int<64>>(1 + 2)"] {
        let inferred = infer(&parse(source), &environment).unwrap();

        assert_eq!(inferred.ty, Type::Int { bits: 64 }, "{source:?}");
    }

    assert_eq!(infer(&parse("wide > 1"), &environment).unwrap().ty, Type::Bool);
}

#[test]
fn numeric_operators_require_matching_types_and_do_not_change_explicit_suffixes() {
    let environment = TypeEnvironment::new();

    for source in ["1 + 2", "-1 * +2", "1.5 / 2.0", "1u32 % 2u32"] {
        assert!(infer(&parse(source), &environment).unwrap().ty.is_numeric());
    }

    for source in ["1i32 + 2i64", "true + 1", "!1", "-1u32", "<Bool>1", "<Int<64>>1u32", "<Int>1.5"] {
        assert!(infer(&parse(source), &environment).is_err(), "{source:?}");
    }
}

#[test]
fn invalid_widths_are_semantic_errors_with_source_ranges() {
    for source in ["1i0", "1f99999999999999999999", "<Int<0>>1", "<Int<true>>1", "<Int<32, 64>>1"] {
        let error = infer(&parse(source), &TypeEnvironment::new()).unwrap_err();

        assert_eq!(error.kind, TypeErrorKind::InvalidWidth, "{source:?}");
        assert!(error.span.end <= source.len());
    }
}

#[test]
fn float_context_and_numeric_type_names_resolve_without_conversions() {
    let environment = TypeEnvironment::from([("value", Type::Float { bits: 32 })]);

    for source in ["value + 1.5", "1 + value", "<Float<32>>1.5", "<Float<32>>(1 + 2)"] {
        let inferred = infer(&parse(source), &environment).unwrap();

        assert_eq!(inferred.ty, Type::Float { bits: 32 }, "{source:?}");
    }

    assert_eq!(infer(&parse("<UInt>1"), &environment).unwrap().ty, Type::UInt { bits: 32 });
    assert_eq!(infer(&parse("<Float>1"), &environment).unwrap().ty, Type::Float { bits: 64 });
}
