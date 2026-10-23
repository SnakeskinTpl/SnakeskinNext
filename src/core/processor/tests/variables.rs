use crate::core::directives::{End, If, Let, Var};
use crate::core::ir::Node;
use crate::core::syntax::document::document;
use crate::core::syntax::literal::LiteralKind;
use crate::core::types::{Type, TypeError, TypeErrorKind};
use crate::parsec::{Parser, ParserInput};

use super::super::Processor;
use super::super::err::{ProcessError, ProcessErrorKind};

fn processor<'a>() -> Processor<'a> {
    let mut processor = Processor::new();
    processor.register_directive("let", Box::new(Let::new())).unwrap();
    processor.register_directive("var", Box::new(Var::new())).unwrap();
    processor.register_directive("if", Box::new(If::new())).unwrap();
    processor.register_directive("end", Box::new(End::new())).unwrap();
    processor
}

#[test]
fn declarations_preserve_initializer_types_mutability_and_byte_ranges() {
    let source = "{let flag = true}{var count = 1u64}{let text = \"hello\"}";
    let mut processor = processor();
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

    let result = processor.process(syntax).unwrap();

    assert_eq!(result.nodes.len(), 3);
    for (node, name, mutable, ty) in [
        (&result.nodes[0], "flag", false, Type::Bool),
        (&result.nodes[1], "count", true, Type::UInt { bits: 64 }),
        (&result.nodes[2], "text", false, Type::Str),
    ] {
        let Node::Declaration { span, declaration } = node else { panic!("Expected declaration") };

        assert_eq!(declaration.name, name);
        assert_eq!(declaration.mutable, mutable);
        assert_eq!(declaration.value_info.ty, ty);
        assert!(declaration.value_info.literal.is_some());
        assert_eq!(&source[declaration.name_span.clone()], name);
        assert!(span.start < declaration.name_span.start);
        assert!(declaration.initializer.span.end < span.end);
    }
}

#[test]
fn declared_variables_supply_context_to_subsequent_initializers_and_conditions() {
    let source = "{let count = 1i64}{var next = 1 + count}{let enabled = next > count}{if enabled}yes{/if}";
    let mut processor = processor();
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

    let result = processor.process(syntax).unwrap();
    let Node::Declaration { declaration, .. } = &result.nodes[1] else { panic!("Expected declaration") };

    assert_eq!(declaration.value_info.ty, Type::Int { bits: 64 });
    assert!(declaration.value_info.literal.is_none());
    assert!(matches!(result.nodes[3], Node::If { .. }));
}

#[test]
fn inner_bindings_shadow_outer_names_and_the_outer_type_returns_after_close() {
    for source in [
        "{let flag = true}{if flag}{var flag = 1}{let next = flag + 1}{/if}{if flag}outer{/if}",
        "- let flag = true\n- if flag\n  - var flag = 1\n  - let next = flag + 1\n- if flag\n  outer",
        "- let flag = true\n- if flag\n  - var flag = 1\n- end if\n- if flag\n  outer",
    ] {
        let mut processor = processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        let result = processor.process(syntax).unwrap();

        assert_eq!(result.nodes.iter().filter(|node| matches!(node, Node::If { .. })).count(), 2, "{source:?}");
    }
}

#[test]
fn shadowing_initializer_can_read_the_outer_binding() {
    let source = "{let value = 1i64}{if true}{var value = value + 1}{/if}";
    let mut processor = processor();
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

    let result = processor.process(syntax).unwrap();
    let Node::If { children, .. } = &result.nodes[1] else { panic!("Expected conditional") };
    let Node::Declaration { declaration, .. } = &children[0] else { panic!("Expected declaration") };

    assert_eq!(declaration.value_info.ty, Type::Int { bits: 64 });
}

#[test]
fn duplicate_names_in_one_scope_are_rejected_at_the_new_name() {
    for source in [
        "{let value = 1}{var value = 2}",
        "{var value = 1}{let value = 2}",
        "{if true}{let value = 1}{let value = 2}{/if}",
    ] {
        let mut processor = processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        let error = processor.process(syntax).unwrap_err();
        let cause = (&*error as &dyn std::error::Error).downcast_ref::<ProcessError>().unwrap();

        assert!(matches!(&cause.kind, ProcessErrorKind::DuplicateVariable { name } if name == "value"));
        assert_eq!(error.location().failure_byte, source.rfind("value").unwrap());
    }
}

#[test]
fn inner_bindings_do_not_escape_explicit_or_inferred_blocks() {
    for source in [
        "{if true}{let local = true}{/if}{if local}{/if}",
        "- if true\n  - let local = true\n- if local\n  text",
        "- if true\n  - let local = true\n- end if\n- if local\n  text",
    ] {
        let mut processor = processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        let error = processor.process(syntax).unwrap_err();
        let cause = error.source().unwrap().downcast_ref::<TypeError>().unwrap();

        assert_eq!(cause.kind, TypeErrorKind::UnknownIdentifier { name: "local".into() });
        assert_eq!(error.location().failure_byte, source.rfind("local").unwrap());
    }
}

#[test]
fn malformed_declarations_and_unknown_initializers_are_rejected() {
    for source in [
        "{let}", "{var value}", "{let = 1}", "{let value =}",
        "{var value: Int = 1}", "{let value = true extra}",
        "{let value = value}", "{var value = missing}",
    ] {
        let mut processor = processor();
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        assert!(processor.process(syntax).is_err(), "{source:?}");
    }
}

#[test]
fn file_bindings_do_not_leak_after_success_or_failure() {
    let mut processor = processor();
    processor.context.types.insert("external", Type::Bool);

    for source in ["{let local = true}", "{let local = true}{if true}{var temporary = 1}{if missing}"] {
        let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

        let _ = processor.process(syntax);

        assert_eq!(processor.context.types.get("local"), None);
        assert_eq!(processor.context.types.get("temporary"), None);
        assert_eq!(processor.context.types.get("external"), Some(&Type::Bool));
        assert_eq!(processor.context.types.len(), 1);
    }

    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new("{if external}ok{/if}")).unwrap();

    assert!(processor.process(syntax).is_ok());
}

#[test]
fn false_blocks_keep_declarations_in_ir_without_exposing_them_outside() {
    let source = "{if false}{let local = true}{/if}";
    let mut processor = processor();
    let (syntax, _) = document(Some(&processor.context)).parse(ParserInput::new(source)).unwrap();

    let result = processor.process(syntax).unwrap();
    let Node::If { children, .. } = &result.nodes[0] else { panic!("Expected conditional") };
    let Node::Declaration { declaration, .. } = &children[0] else { panic!("Expected declaration") };

    assert_eq!(declaration.value_info.literal, Some(LiteralKind::Bool(true)));
    assert_eq!(processor.context.types.get("local"), None);
}
