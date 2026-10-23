use super::*;

fn assert_no_progress(error: &(dyn ParseError + 'static)) {
    assert!(!error.is_recoverable());

    let error = (error as &dyn std::error::Error).downcast_ref::<RepeatError>().unwrap();

    assert!(matches!(error.kind(), RepeatErrorKind::NoProgress));
}

#[test]
fn optional_repeat_distinguishes_mismatch_from_no_progress() {
    let (output, remaining) = opt(repeat(tag("a"), 2..))
        .parse(ParserInput::new("ax"))
        .unwrap();

    assert_eq!(output, None);
    assert_eq!(remaining.byte_position(), 0);

    let error = opt(repeat(succeed(), ..)).parse(ParserInput::new("x")).unwrap_err();

    assert_no_progress(error.as_ref());
}

#[test]
fn negative_lookahead_propagates_non_recoverable_error() {
    let error = not(repeat(succeed(), ..)).parse(ParserInput::new("x")).unwrap_err();

    assert_no_progress(error.as_ref());
}

#[test]
fn alternatives_do_not_try_second_branch_after_non_recoverable_error() {
    let calls = std::cell::Cell::new(0);
    let second = || map(tag("x"), |_| { calls.set(calls.get() + 1); });

    let error = or(repeat(succeed(), ..), second())
        .parse(ParserInput::new("x"))
        .unwrap_err();

    assert_no_progress(error.as_ref());
    assert_eq!(calls.get(), 0);

    let error = choice(void(repeat(succeed(), ..)), second())
        .parse(ParserInput::new("x"))
        .unwrap_err();

    assert_no_progress(error.as_ref());
    assert_eq!(calls.get(), 0);
}

#[test]
fn second_branch_non_recoverable_error_wins_over_further_mismatch() {
    let error = or(tag("abc"), repeat(succeed(), ..))
        .parse(ParserInput::new("abX"))
        .unwrap_err();

    assert_no_progress(error.as_ref());
    assert_eq!(error.byte_position(), 0);

    let error = choice(void(tag("abc")), void(repeat(succeed(), ..)))
        .parse(ParserInput::new("abX"))
        .unwrap_err();

    assert_no_progress(error.as_ref());
}

#[test]
fn nested_repeat_propagates_non_recoverable_error_regardless_of_minimum() {
    for min in [0, 1] {
        let error = repeat(repeat(succeed(), ..), min..)
            .parse(ParserInput::new("x"))
            .unwrap_err();

        assert_no_progress(error.as_ref());
    }

    let inner = flat_map(tag("a"), |_, input| repeat(succeed(), ..).parse(input));
    let parser = repeat(choice(super::value(tag("x"), vec![]), inner), 1..);

    let error = parser.parse(ParserInput::new("xa")).unwrap_err();

    assert_no_progress(error.as_ref());
    assert_eq!(error.byte_position(), 2);
}

#[test]
fn error_wrappers_preserve_non_recoverable_classification() {
    let location = ErrorLocation { start_byte: 0, failure_byte: 0, found_char: Some('x') };
    let mismatch = || TagError::new(location, "a", 'a');
    let fatal = || RepeatError::no_progress(location);

    assert!(mismatch().is_recoverable());
    assert!(!RepeatError::insufficient_matches(location, 1, 0, fatal()).is_recoverable());
    assert!(RepeatError::insufficient_matches(location, 1, 0, mismatch()).is_recoverable());

    assert!(!AlternativeError::new(0, mismatch(), fatal()).is_recoverable());
    assert!(!AlternativeError::new(0, fatal(), mismatch()).is_recoverable());
    assert!(AlternativeError::new(0, mismatch(), mismatch()).is_recoverable());
}

// Intentionally has no Debug implementation.
struct Value<'a>(&'a str);

fn value<'a>(text: &'a str, input: ParserInput<'a>) -> ParserResult<'a, Value<'a>> {
    Ok((Value(text), input))
}

#[test]
fn boxed_concrete_parser_can_be_used_in_combinators() {
    let parser = seq(Box::new(tag("a")), tag("b"));

    let ((first, second), remaining) = parser.parse(ParserInput::new("ab")).unwrap();

    assert_eq!((first, second), ("a", "b"));
    assert!(remaining.is_at_end());
}

#[test]
fn boxed_trait_object_can_borrow_parser_configuration() {
    let pattern = String::from("hello");
    let parser: Box<dyn Parser<'_, Output = &str> + '_> = Box::new(tag(pattern.as_str()));
    let parser = seq(parser, tag("!"));

    let ((output, _), remaining) = parser.parse(ParserInput::new("hello!")).unwrap();

    assert_eq!(output, "hello");
    assert!(remaining.is_at_end());
}

#[test]
fn combinators_accept_output_without_debug() {
    let parser = choice(flat_map(tag("a"), value), flat_map(tag("b"), value));
    let parser = seq(opt(parser), repeat(flat_map(tag("!"), value), 1..=1));

    let ((first, repeated), remaining) = parser.parse(ParserInput::new("b!")).unwrap();

    assert_eq!(first.unwrap().0, "b");
    assert_eq!(repeated[0].0, "!");
    assert!(remaining.is_at_end());
}
