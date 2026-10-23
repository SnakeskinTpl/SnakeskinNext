use crate::parsec::{Parser, ParserInput};

use super::super::directives::Shorthand;
use super::super::syntax::document::document;
use super::*;

#[test]
fn registers_shorthand_from_a_parsed_document() {
    let source = r#"
- shorthand tag { opening: "<", closing: ">" }
"#;
    let mut processor = Processor::new();

    processor.register_directive("shorthand", Box::new(Shorthand::new())).unwrap();

    let (syntax, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    assert!(remaining.is_at_end());

    processor.process(syntax).unwrap();

    let binding = processor.context.shorthands.get("<").unwrap();

    assert_eq!(binding.target, "tag");
    assert_eq!(binding.closing.as_deref(), Some(">"));
    assert_eq!(binding.span.start, source.find("tag {").unwrap());
    assert_eq!(&source[binding.opening_span.clone()], "\"<\"");
}

#[test]
fn unknown_directive_reports_its_original_position_without_panicking() {
    let source = "я\n  - missing value";
    let mut processor = Processor::new();
    let (syntax, _) = document(None).parse(ParserInput::new(source)).unwrap();

    let error = processor.process(syntax).unwrap_err();

    assert!(!error.is_recoverable());
    assert_eq!(error.location().start_byte, source.find("  -").unwrap());
    assert_eq!(error.location().failure_byte, source.find("missing").unwrap());
    assert_eq!(error.location().found_char, Some('m'));

    let error = (&*error as &dyn std::error::Error).downcast_ref::<ProcessError>().unwrap();

    assert!(matches!(&error.kind, ProcessErrorKind::UnknownDirective { name } if name == "missing"));
}

#[test]
fn unresolved_candidates_are_not_silently_skipped() {
    let source = "{< .foo}";
    let mut processor = Processor::new();
    let (syntax, _) = document(None).parse(ParserInput::new(source)).unwrap();

    let error = processor.process(syntax).unwrap_err();
    let error = (&*error as &dyn std::error::Error).downcast_ref::<ProcessError>().unwrap();

    assert!(matches!(error.kind, ProcessErrorKind::UnresolvedDirective));
    assert_eq!(error.location().failure_byte, 1);
}

#[test]
fn missing_required_body_is_reported_before_calling_the_handler() {
    let source = "{shorthand }";
    let mut processor = Processor::new();
    processor.register_directive("shorthand", Box::new(Shorthand::new())).unwrap();
    let (syntax, _) = document(None).parse(ParserInput::new(source)).unwrap();

    let error = processor.process(syntax).unwrap_err();
    let error = (&*error as &dyn std::error::Error).downcast_ref::<ProcessError>().unwrap();

    assert!(matches!(&error.kind, ProcessErrorKind::MissingBody { name } if name == "shorthand"));
    assert_eq!(error.location().failure_byte, source.find('}').unwrap());
    assert!(processor.context.shorthands.is_empty());
}

#[test]
fn handler_errors_keep_absolute_positions_for_both_syntax_forms() {
    for source in ["я\n{shorthand tag {opening: 1}}", "я\n  - shorthand tag {opening: 1}"] {
        let mut processor = Processor::new();
        processor.register_directive("shorthand", Box::new(Shorthand::new())).unwrap();
        let (syntax, _) = document(None).parse(ParserInput::new(source)).unwrap();

        let error = processor.process(syntax).unwrap_err();

        assert_eq!(error.location().failure_byte, source.find('1').unwrap());
        assert_eq!(error.location().found_char, Some('1'));
        assert!(processor.context.shorthands.is_empty());
    }
}

#[test]
fn duplicate_registration_preserves_the_original_handler() {
    let mut processor = Processor::new();
    processor.register_directive("shorthand", Box::new(Shorthand::new())).unwrap();
    let original = processor.directives["shorthand"].as_ref() as *const dyn Directive;

    let error = processor.register_directive("shorthand", Box::new(Shorthand::new())).unwrap_err();

    assert_eq!(error.name, "shorthand");
    assert_eq!(processor.directives.len(), 1);
    assert!(std::ptr::eq(original, processor.directives["shorthand"].as_ref()));
}

#[test]
fn duplicate_shorthand_keeps_original_declaration_ranges() {
    let source = "- shorthand tag {opening: \"<\"}\n- shorthand output {opening: \"<\"}";
    let mut processor = Processor::new();
    processor.register_directive("shorthand", Box::new(Shorthand::new())).unwrap();
    let (syntax, _) = document(None).parse(ParserInput::new(source)).unwrap();

    let error = processor.process(syntax).unwrap_err();

    assert_eq!(error.location().failure_byte, source.rfind("\"<\"").unwrap());
    assert_eq!(processor.context.shorthands.get("<").unwrap().target, "tag");
    assert_eq!(processor.context.shorthands.get("<").unwrap().span,
        source.find("tag {").unwrap()..source.find('\n').unwrap());
}
