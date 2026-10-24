use std::cell::Cell;
use std::rc::Rc;

use crate::core::directives::{
    Directive, DirectiveAction, DirectiveBeginResult, DirectiveContext, DirectiveDefinition,
    DirectiveResult, End, Let,
};

use crate::core::ir::{BlockKind, Node};
use crate::core::syntax::document::document;
use crate::core::syntax::expression::expression;
use crate::core::types::{Binding, Type};
use crate::parsec::{DataError, ErrorLocation, Parser, ParserInput};

use super::super::err::{ProcessError, ProcessErrorKind};
use super::super::Processor;

struct InvalidActionDirective {
    definition: DirectiveDefinition<'static>,
    closes: bool,
    completions: Rc<Cell<usize>>,
}

impl Directive for InvalidActionDirective {
    fn definition(&self) -> &DirectiveDefinition<'_> { &self.definition }

    fn begin<'a>(&self, _: &mut DirectiveContext<'a>, body: ParserInput<'a>) -> DirectiveBeginResult<'a> {
        if self.closes {
            Ok(DirectiveAction::CloseBlock { name: None })
        } else {
            let (condition, _) = expression().parse(body)?;
            Ok(DirectiveAction::OpenBlock { kind: BlockKind::If { condition } })
        }
    }

    fn end(&self, _: &mut DirectiveContext<'_>) -> DirectiveResult {
        self.completions.set(self.completions.get() + 1);
        Ok(())
    }
}

#[test]
fn contradictory_block_actions_fail_instead_of_dropping_ir_or_closing_themselves() {
    for (source, block, closes) in [("{broken true}", false, false), ("{broken}", true, true)] {
        let completions = Rc::new(Cell::new(0));
        let mut processor = Processor::new();
        processor.register_directive("broken", Box::new(InvalidActionDirective {
            definition: DirectiveDefinition { name: "broken", block, ..Default::default() },
            closes,
            completions: completions.clone(),
        })).unwrap();
        let (syntax, _) = document(Some(&processor.context.shorthands)).parse(ParserInput::new(source)).unwrap();

        let error = processor.process(syntax).unwrap_err();
        let cause = (&*error as &dyn std::error::Error).downcast_ref::<ProcessError>().unwrap();

        assert!(matches!(&cause.kind, ProcessErrorKind::InvalidDirectiveAction { name, .. } if name == "broken"));
        assert_eq!(error.location().failure_byte, 1);
        assert_eq!(completions.get(), 0);
        assert_eq!(processor.context.types.len(), 0);
    }
}

struct ScopeObserver {
    definition: DirectiveDefinition<'static>,
    completions: Rc<Cell<usize>>,
    fail: bool,
}

impl Directive for ScopeObserver {
    fn definition(&self) -> &DirectiveDefinition<'_> { &self.definition }

    fn begin<'a>(&self, _: &mut DirectiveContext<'a>, _: ParserInput<'a>) -> DirectiveBeginResult<'a> {
        Ok(DirectiveAction::None)
    }

    fn end(&self, context: &mut DirectiveContext<'_>) -> DirectiveResult {
        assert_eq!(context.types.get("local"), Some(&Type::Bool));
        self.completions.set(self.completions.get() + 1);

        if self.fail {
            Err(DataError::new(ErrorLocation { start_byte: 0, failure_byte: 0, found_char: None }))
        } else {
            Ok(())
        }
    }
}

fn observer_processor(fail: bool) -> (Processor<'static>, Rc<Cell<usize>>) {
    let completions = Rc::new(Cell::new(0));
    let mut processor = Processor::new();
    processor.context.types.insert("local", Binding { ty: Type::Int { bits: 64 }, mutable: false });
    processor.register_directive("let", Box::new(Let::new())).unwrap();
    processor.register_directive("end", Box::new(End::new())).unwrap();
    processor.register_directive("observe", Box::new(ScopeObserver {
        definition: DirectiveDefinition { name: "observe", block: true, ..Default::default() },
        completions: completions.clone(),
        fail,
    })).unwrap();
    (processor, completions)
}

#[test]
fn all_close_paths_run_end_once_in_the_inner_scope_then_restore_the_outer_binding() {
    for source in [
        "{observe}{let local = true}{/observe}{let after = local + 1}",
        "- observe\n  - let local = true\n- let after = local + 1",
        "- observe\n  - let local = true\n- end observe\n- let after = local + 1",
    ] {
        let (mut processor, completions) = observer_processor(false);
        let (syntax, _) = document(Some(&processor.context.shorthands)).parse(ParserInput::new(source)).unwrap();

        let ir = processor.process(syntax).unwrap();
        let after = ir.nodes.iter().find_map(|node| match node {
            Node::Declaration { declaration, .. } if declaration.name == "after" => Some(declaration),
            _ => None,
        }).expect("Expected declaration after the block");

        assert_eq!(after.value_info.ty, Type::Int { bits: 64 });
        assert_eq!(completions.get(), 1, "{source:?}");
        assert_eq!(processor.context.types.get("local"), Some(&Type::Int { bits: 64 }));
    }
}

#[test]
fn failing_end_restores_the_file_environment_for_explicit_and_inferred_closes() {
    for source in ["{observe}{let local = true}{/observe}", "- observe\n  - let local = true"] {
        let (mut processor, completions) = observer_processor(true);
        let (syntax, _) = document(Some(&processor.context.shorthands)).parse(ParserInput::new(source)).unwrap();

        assert!(processor.process(syntax).is_err());

        assert_eq!(completions.get(), 1);
        assert_eq!(processor.context.types.len(), 1);
        assert_eq!(processor.context.types.get("local"), Some(&Type::Int { bits: 64 }));
    }
}
