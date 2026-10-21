use std::any::Any;

use super::{ParserInput, ParserState};

#[derive(Clone, Debug, PartialEq)]
struct TextState(String);

impl ParserState for TextState {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ParserState> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
struct OtherState;

impl ParserState for OtherState {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ParserState> {
        Box::new(self.clone())
    }
}

#[test]
fn empty_input_stays_at_end() {
    let mut input = ParserInput::from("");

    assert_eq!(input.byte_position(), 0);
    assert!(input.is_at_end());
    assert_eq!(input.peek(), None);
    assert_eq!(input.next(), None);
    assert_eq!(input.next(), None);
    assert_eq!(input.byte_position(), 0);
}

#[test]
fn unicode_iteration_tracks_byte_positions_and_peek_does_not_advance() {
    let mut input = ParserInput::new("aя€🦀");

    for (ch, start, end) in [('a', 0, 1), ('я', 1, 3), ('€', 3, 6), ('🦀', 6, 10)] {
        assert!(!input.is_at_end());
        assert_eq!(input.byte_position(), start);

        assert_eq!(input.peek(), Some(ch));
        assert_eq!(input.peek(), Some(ch));
        assert_eq!(input.byte_position(), start);

        assert_eq!(input.next(), Some(ch));
        assert_eq!(input.byte_position(), end);
    }

    assert!(input.is_at_end());
    assert_eq!(input.peek(), None);
    assert_eq!(input.next(), None);
    assert_eq!(input.next(), None);
    assert_eq!(input.byte_position(), 10);
}

#[test]
fn position_can_move_forward_backward_and_back_from_end() {
    let mut input = ParserInput::new("aя🦀");
    input.set_byte_position(3);

    assert_eq!(input.next(), Some('🦀'));
    assert!(input.is_at_end());

    input.set_byte_position(1);

    assert_eq!(input.next(), Some('я'));

    input.set_byte_position(7);

    assert_eq!(input.next(), None);

    input.set_byte_position(0);

    assert_eq!(input.next(), Some('a'));
}

#[test]
#[should_panic]
fn position_inside_utf8_character_is_rejected() {
    ParserInput::new("я").set_byte_position(1);
}

#[test]
#[should_panic]
fn position_past_end_is_rejected() {
    ParserInput::new("a").set_byte_position(2);
}

#[test]
fn states_are_lifo_and_only_the_current_state_is_matched() {
    let mut input = ParserInput::new("");
    let state = TextState("outer".into());

    assert!(input.peek_state().is_none());
    assert!(input.pop_state().is_none());
    assert!(!input.is_current_state(&state));

    input.push_state(state.clone());

    assert!(input.is_current_state(&state));
    assert!(!input.is_current_state(&TextState("different".into())));

    input.push_state(OtherState);

    assert!(!input.is_current_state(&state));
    assert!(input.is_current_state(&OtherState));
    assert!(input.peek_state().unwrap().as_any().is::<OtherState>());

    assert!(input.pop_state().unwrap().as_any().is::<OtherState>());
    assert!(input.is_current_state(&state));

    let popped = input.pop_state().unwrap();

    assert_eq!(popped.as_any().downcast_ref::<TextState>(), Some(&state));
    assert!(input.peek_state().is_none());
}

#[test]
fn taking_and_replacing_states_preserves_order_and_returns_previous_stack() {
    let mut input = ParserInput::new("");
    let outer = TextState("outer".into());
    input.push_state(outer.clone());
    input.push_state(OtherState);

    let saved = input.take_states();

    assert!(input.peek_state().is_none());
    assert!(input.take_states().is_empty());

    let temporary = TextState("temporary".into());
    input.push_state(temporary.clone());
    let previous = input.replace_states(saved);

    assert_eq!(previous.len(), 1);
    assert_eq!(previous[0].as_any().downcast_ref::<TextState>(), Some(&temporary));
    assert!(input.is_current_state(&OtherState));

    input.pop_state();

    assert!(input.is_current_state(&outer));

    let previous = input.replace_states(vec![]);

    assert_eq!(previous.len(), 1);
    assert!(input.peek_state().is_none());
}

#[test]
fn clone_preserves_position_and_independently_owned_states() {
    let mut original = ParserInput::new("яa");
    original.next();
    original.push_state(OtherState);
    original.push_state(TextState("original".into()));

    let mut cloned = original.clone();

    assert_eq!(cloned.byte_position(), 2);
    assert!(cloned.is_current_state(&TextState("original".into())));

    assert_eq!(cloned.next(), Some('a'));
    assert_eq!(original.peek(), Some('a'));
    assert_eq!(original.byte_position(), 2);

    let mut state = cloned.pop_state().unwrap();
    state.as_any_mut().downcast_mut::<TextState>().unwrap().0.push_str(" changed");

    assert!(original.is_current_state(&TextState("original".into())));
    assert!(cloned.is_current_state(&OtherState));
    assert_eq!(state.as_any().downcast_ref::<TextState>().unwrap().0, "original changed");

    original.pop_state();

    assert!(original.is_current_state(&OtherState));
}
