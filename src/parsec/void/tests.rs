use crate::parsec::{cut, parser, tag, ParserInput, ParserResult};

use super::*;

#[test]
fn discards_output_and_preserves_consumed_unicode_range() {
    let input = ParserInput::with_offset("\u{0436}tail", 12);

    let ((), remaining) = void(tag("\u{0436}")).parse(input).unwrap();

    assert_eq!(remaining.byte_position(), 14);
    assert_eq!(remaining.peek(), Some('t'));
}

#[test]
fn preserves_the_original_error_and_recoverability() {
    for committed in [false, true] {
        let input = ParserInput::with_offset("ab!", 8);
        let inner = parser(move |input| {
            if committed {
                cut(tag("abc")).parse(input)
            } else {
                tag("abc").parse(input)
            }
        });
        let original = inner.parse(input.clone()).unwrap_err();

        let error = void(inner).parse(input).unwrap_err();

        assert_eq!(error.location(), original.location());
        assert_eq!(error.is_recoverable(), original.is_recoverable());
        assert_eq!(error.to_string(), original.to_string());
    }
}

#[test]
fn accepts_output_without_debug_or_clone_and_preserves_empty_success() {
    struct Output;
    let inner = parser(|input| -> ParserResult<'_, Output> { Ok((Output, input)) });

    let ((), remaining) = void(inner).parse(ParserInput::new("text")).unwrap();

    assert_eq!(remaining.byte_position(), 0);
    assert_eq!(remaining.peek(), Some('t'));
}
