use std::cell::Cell;
use std::rc::Rc;

use crate::parsec::{cut, repeat, tag, ParserInput};

use super::*;

#[test]
fn repeated_successes_return_independent_owned_values_and_consume_input() {
    let parser = repeat(value(tag("x"), String::from("item")), 1..);

    let (mut items, remaining) = parser.parse(ParserInput::new("xx!")).unwrap();
    items[0].push('!');

    assert_eq!(items, vec!["item!", "item"]);
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn clones_only_after_success_and_can_be_reused_after_failure() {
    struct Output(Rc<Cell<usize>>);
    impl Clone for Output {
        fn clone(&self) -> Self {
            self.0.set(self.0.get() + 1);
            Self(self.0.clone())
        }
    }
    let clones = Rc::new(Cell::new(0));
    let parser = value(tag("x"), Output(clones.clone()));

    let result = parser.parse(ParserInput::new("!"));

    assert!(result.is_err());
    assert_eq!(clones.get(), 0);

    let (_, remaining) = parser.parse(ParserInput::new("x")).unwrap();

    assert_eq!(clones.get(), 1);
    assert!(remaining.is_at_end());
}

#[test]
fn preserves_committed_errors_and_absolute_positions() {
    let input = ParserInput::with_offset("ab!", 9);
    let original = cut(tag("abc")).parse(input.clone()).unwrap_err();

    let error = value(cut(tag("abc")), 42).parse(input).unwrap_err();

    assert_eq!(error.location(), original.location());
    assert_eq!(error.to_string(), original.to_string());
    assert!(!error.is_recoverable());
}
