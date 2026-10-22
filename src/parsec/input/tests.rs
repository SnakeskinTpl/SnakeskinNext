use super::ParserInput;

#[test]
fn offset_input_preserves_source_positions_and_bounds() {
    let mut input = ParserInput::with_offset("я🦀", 10);

    assert_eq!(input.byte_position(), 10);
    assert!(!input.is_at_start());
    assert_eq!(input.peek_prev(), None);

    assert_eq!(input.next(), Some('я'));
    assert_eq!(input.byte_position(), 12);
    assert_eq!(input.source_slice(10..12), "я");

    let mut cloned = input.clone();
    cloned.set_byte_position(16);

    assert!(cloned.is_at_end());
    assert_eq!(cloned.byte_position(), 16);
    assert_eq!(input.peek(), Some('🦀'));

    cloned.set_byte_position(10);

    assert_eq!(cloned.peek(), Some('я'));
    assert_eq!(cloned.source_slice(10..16), "я🦀");
}

#[test]
fn parsing_offset_input_returns_absolute_spans_and_errors() {
    use crate::parsec::{spanned, tag, Parser};

    let input = ParserInput::with_offset("я!", 20);

    let ((span, text), remaining) = spanned(tag("я")).parse(input.clone()).unwrap();

    assert_eq!(span, 20..22);
    assert_eq!(text, "я");
    assert_eq!(remaining.peek(), Some('!'));

    let error = tag("я?").parse(input).unwrap_err();

    assert_eq!(error.location().start_byte, 20);
    assert_eq!(error.location().failure_byte, 22);
    assert_eq!(error.location().found_char, Some('!'));
}

#[test]
#[should_panic]
fn offset_input_cannot_seek_before_its_start() {
    ParserInput::with_offset("text", 10).set_byte_position(9);
}

#[test]
#[should_panic]
fn offset_input_cannot_expose_text_outside_its_bounds() {
    ParserInput::with_offset("text", 10).source_slice(10..15);
}

#[test]
fn previous_character_tracks_unicode_cursor_without_moving_it() {
    let mut input = ParserInput::new("aя🦀");

    assert_eq!(input.peek_prev(), None);

    for ch in ['a', 'я', '🦀'] {
        input.next();
        let position = input.byte_position();

        assert_eq!(input.peek_prev(), Some(ch));
        assert_eq!(input.peek_prev(), Some(ch));
        assert_eq!(input.byte_position(), position);
    }

    assert!(input.is_at_end());

    input.set_byte_position(1);

    assert_eq!(input.peek_prev(), Some('a'));
    assert_eq!(input.peek(), Some('я'));

    input.set_byte_position(0);

    assert_eq!(input.peek_prev(), None);
    assert_eq!(ParserInput::new("").peek_prev(), None);
}

#[test]
fn start_predicate_tracks_cursor_including_rewind_and_empty_input() {
    let empty = ParserInput::new("");

    assert!(empty.is_at_start());
    assert!(empty.is_at_end());

    let mut input = ParserInput::new("я");

    assert!(input.is_at_start());

    input.next();

    assert!(!input.is_at_start());
    assert!(input.is_at_end());

    input.set_byte_position(0);

    assert!(input.is_at_start());
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

    for (ch, start_byte, end) in [('a', 0, 1), ('я', 1, 3), ('€', 3, 6), ('🦀', 6, 10)] {
        assert!(!input.is_at_end());
        assert_eq!(input.byte_position(), start_byte);

        assert_eq!(input.peek(), Some(ch));
        assert_eq!(input.peek(), Some(ch));
        assert_eq!(input.byte_position(), start_byte);

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
fn clone_preserves_position_and_moves_independently() {
    let source = "яa";
    let mut original = ParserInput::new(source);
    original.next();

    let mut cloned = original.clone();

    assert_eq!(cloned.byte_position(), 2);
    assert_eq!(cloned.source_slice(0..source.len()).as_ptr(), source.as_ptr());

    assert_eq!(cloned.next(), Some('a'));
    assert!(cloned.is_at_end());
    assert_eq!(original.peek(), Some('a'));
    assert_eq!(original.byte_position(), 2);

    original.set_byte_position(0);

    assert_eq!(original.peek(), Some('я'));
    assert!(cloned.is_at_end());
}
