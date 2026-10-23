use crate::core::directives::DirectiveAction;
use crate::core::ir::Node;
use crate::parsec::{Parser, ParserInput};

use super::super::directives::Shorthand;
use super::super::syntax::document::document;
use super::*;
use super::err::{ProcessError, ProcessErrorKind};

mod variables;

struct RecordingDirective {
    definition: super::super::directives::DirectiveDefinition<'static>,
    events: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
}

impl Directive for RecordingDirective {
    fn definition(&self) -> &super::super::directives::DirectiveDefinition<'_> {
        &self.definition
    }

    fn begin<'a>(
        &self,
        _context: &mut DirectiveContext<'a>,
        _body: ParserInput<'a>,
    ) -> super::super::directives::DirectiveBeginResult<'a> {
        self.events.borrow_mut().push(format!("begin {}", self.definition.name));
        Ok(DirectiveAction::None)
    }

    fn end(&self, _context: &mut DirectiveContext<'_>) -> super::super::directives::DirectiveResult {
        self.events.borrow_mut().push(format!("end {}", self.definition.name));
        Ok(())
    }
}

fn recording_processor() -> (Processor<'static>, std::rc::Rc<std::cell::RefCell<Vec<String>>>) {
    use super::super::directives::{DirectiveDefinition, End};

    let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let mut processor = Processor::new();
    processor.register_directive("end", Box::new(End::new())).unwrap();

    for (name, block) in [("block", true), ("inline", false)] {
        processor.register_directive(name, Box::new(RecordingDirective {
            definition: DirectiveDefinition { name, block, ..DirectiveDefinition::default() },
            events: events.clone(),
        })).unwrap();
    }

    (processor, events)
}

#[test]
fn explicit_jade_like_ends_confirm_dedent_without_closing_the_parent() {
    for source in [
        "- block value\n  text\n- end block",
        "- block value\n  text\n- end",
        "- block value\n  text\n- /block",
        "- block outer\n  - block inner\n    text\n  - end block\n  - inline value",
        "- block outer\n  - block inner\n    text\n- end block",
    ] {
        let (mut processor, events) = recording_processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        processor.process(syntax).unwrap();

        let expected = if source.contains("inline") {
            vec!["begin block", "begin block", "end block", "begin inline", "end inline", "end block"]
        } else if source.contains("inner") {
            vec!["begin block", "begin block", "end block", "end block"]
        } else {
            vec!["begin block", "end block"]
        };

        assert_eq!(*events.borrow(), expected, "{source:?}");
    }
}

#[test]
fn inferred_close_confirmation_checks_names_and_cannot_be_reused() {
    for (source, mismatch) in [
        ("- block value\n- end inline", true),
        ("- block value\n- end block\n- end block", false),
        ("- block value\ntext\n- end block", false),
    ] {
        let (mut processor, events) = recording_processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        let error = processor.process(syntax).unwrap_err();
        let error = (&*error as &dyn std::error::Error).downcast_ref::<ProcessError>().unwrap();

        if mismatch {
            assert!(matches!(&error.kind, ProcessErrorKind::MismatchedEnd { expected, found }
                if expected == "block" && found == "inline"));
        } else {
            assert!(matches!(error.kind, ProcessErrorKind::UnexpectedEnd));
        }

        assert_eq!(*events.borrow(), vec!["begin block", "end block"]);
    }
}

#[test]
fn explicit_and_inferred_ends_call_handlers_in_nesting_order() {
    for source in [
        "{block }{inline }{block }{end block}{/block}",
        "- block \n  - inline \n  - block value",
        "{block }{end}",
        "{block }{/}",
        "{block }\n- /block",
    ] {
        let (mut processor, events) = recording_processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        processor.process(syntax).unwrap();

        let expected = if matches!(source, "{block }{end}" | "{block }{/}" | "{block }\n- /block") {
            vec!["begin block", "end block"]
        } else {
            vec!["begin block", "begin inline", "end inline", "begin block", "end block", "end block"]
        };

        assert_eq!(*events.borrow(), expected);
    }
}

#[test]
fn conflicting_declared_shorthands_do_not_partially_register_a_directive() {
    use super::super::directives::DirectiveDefinition;
    use super::super::shorthands::ShorthandDefinition;

    let (mut processor, events) = recording_processor();
    let original_count = processor.directives.len();
    let original_shorthands = processor.context.shorthands.len();
    let directive = RecordingDirective {
        definition: DirectiveDefinition {
            name: "conflict",
            shorthands: vec![
                ShorthandDefinition { opening: "!", closing: None },
                ShorthandDefinition { opening: "/", closing: None },
            ],
            ..DirectiveDefinition::default()
        },
        events,
    };

    let error = processor.register_directive("conflict", Box::new(directive)).unwrap_err();

    assert_eq!(error.name, "/");
    assert_eq!(processor.directives.len(), original_count);
    assert_eq!(processor.context.shorthands.len(), original_shorthands);
    assert!(processor.context.shorthands.get("!").is_none());
    assert_eq!(processor.context.shorthands.get("/").unwrap().target, "end");
}

#[test]
fn implicit_inline_end_does_not_close_parent_block() {
    let source = "- block value\n  - inline value\n  text";
    let (mut processor, events) = recording_processor();
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

    processor.process(syntax).unwrap();

    assert_eq!(*events.borrow(), vec!["begin block", "begin inline", "end inline", "end block"]);
}

#[test]
fn explicit_close_before_inferred_close_does_not_close_parent_twice() {
    let source = "{block }\n  - block value\n    {/block}\ntext\n{/block}";
    let (mut processor, events) = recording_processor();
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

    processor.process(syntax).unwrap();

    assert_eq!(*events.borrow(), vec!["begin block", "begin block", "end block", "end block"]);
}

#[test]
fn invalid_closures_and_unclosed_blocks_are_reported() {
    for (source, expected) in [
        ("{end}", "unexpected"),
        ("{/block}", "unexpected"),
        ("{block }{end inline}", "mismatch"),
        ("{block }", "unclosed"),
        ("- block value\n  {block }\ntext", "crossed"),
    ] {
        let (mut processor, _) = recording_processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        let error = processor.process(syntax).unwrap_err();
        let error = (&*error as &dyn std::error::Error).downcast_ref::<ProcessError>().unwrap();

        assert!(!error.is_recoverable());
        assert!(match (&error.kind, expected) {
            (ProcessErrorKind::UnexpectedEnd, "unexpected") => true,
            (ProcessErrorKind::MismatchedEnd { expected, found }, "mismatch") => expected == "block" && found == "inline",
            (ProcessErrorKind::UnclosedBlock { name }, "unclosed") => name == "block",
            (ProcessErrorKind::CrossedImplicitEnd, "crossed") => true,
            _ => false,
        }, "{source:?}: {error}");

        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new("{block }{/block}")).unwrap();

        processor.process(syntax).unwrap();
    }
}

#[test]
fn end_rejects_trailing_arguments_at_the_original_position() {
    let source = "{block }{end block extra}";
    let (mut processor, events) = recording_processor();
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

    let error = processor.process(syntax).unwrap_err();

    assert_eq!(error.location().failure_byte, source.find("extra").unwrap());
    assert!(!error.is_recoverable());
    assert_eq!(*events.borrow(), vec!["begin block"]);
}

#[test]
fn registers_shorthand_from_a_parsed_document() {
    let source = r#"
- shorthand tag { opening: "<", closing: ">" }
"#;
    let mut processor = Processor::new();

    processor.register_directive("shorthand", Box::new(Shorthand::new())).unwrap();

    let (syntax, remaining) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

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
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

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
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

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
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

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
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

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
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

    let error = processor.process(syntax).unwrap_err();

    assert_eq!(error.location().failure_byte, source.rfind("\"<\"").unwrap());
    assert_eq!(processor.context.shorthands.get("<").unwrap().target, "tag");
    assert_eq!(processor.context.shorthands.get("<").unwrap().span,
        source.find("tag {").unwrap()..source.find('\n').unwrap());
}

fn conditional_processor() -> Processor<'static> {
    use super::super::directives::{End, If};
    use super::super::types::Type;

    let mut processor = Processor::new();
    processor.register_directive("if", Box::new(If::new())).unwrap();
    processor.register_directive("end", Box::new(End::new())).unwrap();
    processor.context.types.insert("enabled", Type::Bool);
    processor
}

#[test]
fn if_preserves_false_conditions_and_text_without_evaluating_them() {
    use super::super::syntax::expression::ExpressionKind;
    use super::super::syntax::literal::LiteralKind;

    let source = "before{if false}inside{/if}after";
    let mut processor = conditional_processor();
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

    let result = processor.process(syntax).unwrap();

    assert_eq!(result.nodes.len(), 3);
    let Node::If { span, condition, children } = &result.nodes[1] else { panic!("Expected conditional") };

    assert_eq!(*span, 6..27);
    assert_eq!(condition.span, 10..15);
    assert_eq!(condition.kind, ExpressionKind::Literal(LiteralKind::Bool(false)));
    assert_eq!(children.len(), 1);
    assert!(matches!(&children[0], Node::Text { content: "inside", span } if *span == (16..22)));
    assert!(matches!(&result.nodes[0], Node::Text { content: "before", .. }));
    assert!(matches!(&result.nodes[2], Node::Text { content: "after", .. }));
}

#[test]
fn if_supports_nested_classic_and_jade_blocks_with_explicit_or_inferred_ends() {
    for source in [
        "{if enabled}{if !false}inner{/if}{/if}",
        "- if enabled\n  - if !false\n    inner",
        "- if enabled\n  - if !false\n    inner\n  - end if\n- end if",
    ] {
        let mut processor = conditional_processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        let result = processor.process(syntax).unwrap();
        let outer = result.nodes.iter().find_map(|node| match node {
            Node::If { children, .. } => Some(children),
            _ => None,
        }).expect("Expected outer conditional");
        let inner = outer.iter().find_map(|node| match node {
            Node::If { children, .. } => Some(children),
            _ => None,
        }).expect("Expected nested conditional");

        assert!(inner.iter().any(|node| matches!(node, Node::Text { content, .. } if content.contains("inner"))), "{source:?}");
    }
}

#[test]
fn if_requires_a_complete_boolean_condition_with_located_errors() {
    use super::super::types::{Type, TypeError, TypeErrorKind};

    for source in ["{if 1}body{/if}", "- if \"text\"\n  body"] {
        let mut processor = conditional_processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        let error = processor.process(syntax).unwrap_err();
        let cause = error.source().unwrap().downcast_ref::<TypeError>().unwrap();

        assert!(matches!(cause.kind, TypeErrorKind::MismatchedType { expected: Type::Bool, .. }));
        assert_eq!(error.location().failure_byte, source.find("1").or_else(|| source.find('"')).unwrap());
    }
}

#[test]
fn if_rejects_empty_unknown_and_incomplete_conditions_and_unclosed_blocks() {
    for source in ["{if}", "{if missing}{/if}", "{if true extra}{/if}", "{if !}{/if}", "{if true}"] {
        let mut processor = conditional_processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        assert!(processor.process(syntax).is_err(), "{source:?}");
    }
}

#[test]
fn failed_if_processing_does_not_leak_output_or_block_state() {
    let mut processor = conditional_processor();
    let (invalid, _) = document(Some(&processor.context)).parse(ParserInput::new("{if true}{if 1}")).unwrap();

    assert!(processor.process(invalid).is_err());

    let source = "{if enabled}ok{/if}";
    let (valid, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

    let result = processor.process(valid).unwrap();

    assert_eq!(result.nodes.len(), 1);
    assert!(matches!(&result.nodes[0], Node::If { children, .. } if children.len() == 1));
}
