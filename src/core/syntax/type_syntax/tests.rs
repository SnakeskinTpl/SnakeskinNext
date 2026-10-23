use crate::parsec::prelude::*;

use super::super::identifier;
use super::{type_syntax, TypeSyntax, TypeSyntaxKind, TypeArgumentSyntax};
use super::super::literal::LiteralKind;

#[test]
fn reads_builtin_and_unresolved_names_without_resolving_them() {
    for source in ["Bool", "Str", "User", "UnknownType", "Type_2"] {
        let (parsed, remaining) = type_syntax().parse(ParserInput::new(source)).unwrap();

        assert_eq!(parsed, TypeSyntax { span: 0..source.len(), kind: TypeSyntaxKind::Named(source) });
        assert!(remaining.is_at_end());
    }
}

#[test]
fn skips_leading_whitespace_and_preserves_separators() {
    for separator in [",", ")", ">", " = value"] {
        let source = format!("  Bool{separator}");

        let (parsed, remaining) = type_syntax().parse(ParserInput::new(&source)).unwrap();

        assert_eq!(parsed, TypeSyntax { span: 2..6, kind: TypeSyntaxKind::Named("Bool") });
        assert_eq!(remaining.source_slice(6..source.len()), separator);
    }
}

#[test]
fn composes_with_parameter_syntax() {
    let grammar = seq!(identifier(), ltrim(tag(":")), type_syntax());

    let (((name, _), annotation), remaining) = grammar
        .parse(ParserInput::new("flag: Bool)"))
        .unwrap();

    assert_eq!(name, "flag");
    assert_eq!(annotation, TypeSyntax { span: 6..10, kind: TypeSyntaxKind::Named("Bool") });
    assert_eq!(remaining.peek(), Some(')'));
}

#[test]
fn unicode_names_preserve_absolute_byte_ranges() {
    let source = "  Тип_2";

    let (parsed, remaining) = type_syntax()
        .parse(ParserInput::with_offset(source, 20))
        .unwrap();

    assert_eq!(parsed.kind, TypeSyntaxKind::Named( "Тип_2"));
    assert_eq!(parsed.span, 22..20 + source.len());
    assert!(remaining.is_at_end());
}

#[test]
fn requires_a_capitalized_identifier() {
    for source in ["", "  ", "bool", "true", "_Bool", "1Bool", "<Bool>"] {
        let error = type_syntax().parse(ParserInput::new(source)).unwrap_err();

        assert!(error.is_recoverable(), "{source:?}");
    }
}

#[test]
fn parses_multiple_nested_type_arguments_with_absolute_spans() {
    let source = " Res<Str, Opt<Bool>> ";

    let (parsed, remaining) = type_syntax()
        .parse(ParserInput::with_offset(source, 10))
        .unwrap();
    let TypeSyntaxKind::Applied { name, arguments } = parsed.kind else { panic!("Expected applied type") };

    assert_eq!(name, "Res");
    assert_eq!(parsed.span, 11..30);
    assert_eq!(remaining.byte_position(), 30);
    assert_eq!(arguments.len(), 2);
    assert_eq!(arguments[0], TypeArgumentSyntax::Type(TypeSyntax {
        span: 15..18,
        kind: TypeSyntaxKind::Named("Str"),
    }));

    let TypeArgumentSyntax::Type(inner) = &arguments[1] else { panic!("Expected type argument") };
    let TypeSyntaxKind::Applied { name, arguments } = &inner.kind else { panic!("Expected nested type") };

    assert_eq!(inner.span, 20..29);
    assert_eq!(*name, "Opt");
    assert_eq!(arguments[0], TypeArgumentSyntax::Type(TypeSyntax {
        span: 24..28,
        kind: TypeSyntaxKind::Named("Bool"),
    }));
}

#[test]
fn mixes_types_and_literal_arguments_without_checking_their_semantics() {
    for source in ["Array<Int, 3>", "Config<Bool, true, \"text\", null>"] {
        let (parsed, remaining) = type_syntax().parse(ParserInput::new(source)).unwrap();
        let TypeSyntaxKind::Applied { arguments, .. } = parsed.kind else { panic!("Expected applied type") };

        assert!(matches!(arguments[0], TypeArgumentSyntax::Type(_)));
        assert!(matches!(arguments[1], TypeArgumentSyntax::Value(_)));
        assert!(remaining.is_at_end());
    }

    let (parsed, _) = type_syntax().parse(ParserInput::new("Int<32>")).unwrap();
    let TypeSyntaxKind::Applied { arguments, .. } = parsed.kind else { panic!("Expected applied type") };
    let TypeArgumentSyntax::Value(value) = &arguments[0] else { panic!("Expected value argument") };
    let LiteralKind::Number(number) = &value.kind else { panic!("Expected numeric literal") };

    assert_eq!(value.span, 4..6);
    assert_eq!(number.content, "32");
}

#[test]
fn allows_whitespace_between_type_name_and_arguments() {
    let (parsed, remaining) = type_syntax().parse(ParserInput::new(" Opt < Bool >," )).unwrap();
    let TypeSyntaxKind::Applied { arguments, .. } = parsed.kind else { panic!("Expected applied type") };

    assert_eq!(arguments.len(), 1);
    assert_eq!(parsed.span, 1..13);
    assert_eq!(remaining.peek(), Some(','));
}

#[test]
fn incomplete_or_malformed_argument_lists_do_not_fall_back_to_named_types() {
    for source in ["Opt<", "Opt<>", "Opt<Bool", "Res<Str,>", "Res<,Str>", "Res<Str Bool>", "Opt<vec>", "Int<1e>"] {
        let error = type_syntax().parse(ParserInput::new(source)).unwrap_err();

        assert!(!error.is_recoverable(), "{source:?}");
    }
}
