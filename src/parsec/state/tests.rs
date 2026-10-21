use std::cell::Cell;

use crate::parsec::*;

use super::*;

#[test]
fn factory_is_deferred_and_state_is_fresh_after_success_and_failure() {
    let calls = Cell::new(0);
    let parser = state(
        || {
            calls.set(calls.get() + 1);
            0usize
        },
        |count, input| {
            assert_eq!(*count, 0);
            *count += 1;

            let (_, remaining) = tag("x").parse(input)?;
            Ok((*count, remaining))
        },
    );

    assert_eq!(calls.get(), 0);

    let (output, _) = parser.parse(ParserInput::new("x")).unwrap();

    assert_eq!(output, 1);
    assert_eq!(calls.get(), 1);

    let error = parser.parse(ParserInput::new("z")).unwrap_err();

    assert_eq!(error.byte_position(), 0);
    assert_eq!(calls.get(), 2);

    let (output, _) = parser.parse(ParserInput::new("x")).unwrap();

    assert_eq!(output, 1);
    assert_eq!(calls.get(), 3);
}

#[test]
fn callback_receives_current_input_and_can_return_borrowed_output() {
    let source = String::from("!я🦀rest");
    let mut input = ParserInput::new(&source);
    input.set_byte_position(1);
    let parser = state(|| (), |_, input| tag("я🦀").parse(input));

    let (output, remaining) = parser.parse(input).unwrap();

    assert_eq!(output, "я🦀");
    assert_eq!(output.as_ptr(), source[1..].as_ptr());
    assert_eq!(remaining.byte_position(), 7);
    assert_eq!(remaining.peek(), Some('r'));
}

#[test]
fn callback_error_preserves_context_and_recoverability() {
    let parser = state(|| (), |_, input| repeat(succeed(), ..).parse(input));

    let error = parser.parse(ParserInput::new("x")).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation { start_byte: 0, failure_byte: 0, found_char: Some('x') });
    assert!(!error.is_recoverable());
    assert!((&*error as &dyn std::error::Error).is::<RepeatError>());
}

#[test]
fn local_counter_tracks_nested_braces() {
    let parser = state(|| Cell::new(0usize), |depth, input| {
        let content = take(|ch, escaped| {
            if escaped {
                return true;
            }

            match ch {
                '{' => depth.set(depth.get() + 1),
                '}' if depth.get() > 0 => depth.set(depth.get() - 1),
                '}' => return false,
                _ => {}
            }

            true
        }, ..);

        let ((_, content), remaining) = seq(tag("{"), content).parse(input)?;
        let (_, remaining) = tag("}").parse(remaining)?;

        Ok((content, remaining))
    });

    let (output, remaining) = parser.parse(ParserInput::new("{a{b}c}!")).unwrap();

    assert_eq!(output, "a{b}c");
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn backtracking_inside_callback_does_not_restore_local_state() {
    let parser = state(|| Cell::new(0usize), |count, input| {
        let first = flat_map(tag("a"), |_, input| {
            count.set(count.get() + 1);
            tag("x").parse(input)
        });

        let (_, remaining) = choice(first, tag("ab")).parse(input)?;

        Ok((count.get(), remaining))
    });

    let (output, remaining) = parser.parse(ParserInput::new("ab")).unwrap();

    assert_eq!(output, 1);
    assert!(remaining.is_at_end());
}
