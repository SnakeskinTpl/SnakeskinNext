use super::*;

#[test]
fn output_borrows_source_and_outlives_parser() {
    let source = String::from("!я🦀rest");

    let output = {
        let parser = tag(String::from("я🦀"));
        let mut input = ParserInput::new(&source);
        input.set_byte_position(1);

        let (output, _) = parser.parse(input).unwrap();
        output
    };

    assert_eq!(output, "я🦀");
    assert_eq!(output.as_ptr(), source[1..].as_ptr());
}

#[test]
fn combinators_preserve_borrowed_outputs() {
    let source = String::from("ab");
    let parser = seq(tag("a"), opt(choice(tag("x"), tag("b"))));

    let ((first, second), remaining) = parser.parse(ParserInput::new(&source)).unwrap();

    assert_eq!(first, "a");
    assert_eq!(second, Some("b"));
    assert_eq!(first.as_ptr(), source.as_ptr());
    assert_eq!(second.unwrap().as_ptr(), source[1..].as_ptr());
    assert!(remaining.is_at_end());
}

#[test]
fn matches_exact_tag() {
    let (output, remaining) = tag(String::from("hello"))
        .parse(ParserInput::new("hello"))
        .unwrap();

    assert_eq!(output, "hello");
    assert_eq!(remaining.byte_position(), 5);
    assert!(remaining.is_at_end());
}

#[test]
fn matches_unicode_prefix_from_current_byte_position() {
    let mut input = ParserInput::new("!я🦀rest");
    input.set_byte_position(1);

    let (output, remaining) = tag("я🦀").parse(input).unwrap();

    assert_eq!(output, "я🦀");
    assert_eq!(remaining.byte_position(), 7);
    assert_eq!(remaining.peek(), Some('r'));
}

#[test]
fn mismatch_reports_expected_and_actual_characters() {
    let mut input = ParserInput::new("!яx");
    input.set_byte_position(1);

    let error = tag("я🦀").parse(input).unwrap_err();

    assert_eq!(error.byte_position(), 3);

    let error = (&*error as &dyn std::error::Error).downcast_ref::<TagError>().unwrap();

    assert_eq!(error.location().start, 1);
    assert_eq!(error.expected_tag, "я🦀");
    assert_eq!(error.expected, '🦀');
    assert_eq!(error.location().actual, Some('x'));
    assert!(error.to_string().contains('x'));
}

#[test]
fn incomplete_tag_preserves_expectation_at_end_of_input() {
    let error = tag("я🦀").parse(ParserInput::new("я")).unwrap_err();

    assert_eq!(error.byte_position(), 2);

    let error = (&*error as &dyn std::error::Error).downcast_ref::<TagError>().unwrap();

    assert_eq!(error.location().start, 0);
    assert_eq!(error.expected_tag, "я🦀");
    assert_eq!(error.expected, '🦀');
    assert_eq!(error.location().actual, None);
    assert!(error.to_string().contains("end of input"));
}

#[test]
fn nonempty_tag_rejects_empty_input() {
    let error = tag("a").parse(ParserInput::new("")).unwrap_err();

    assert_eq!(error.byte_position(), 0);
}

#[test]
fn empty_tag_and_succeed_do_not_consume_input() {
    for source in ["", "я"] {
        for parser in [tag(""), succeed()] {
            let (output, remaining) = parser.parse(ParserInput::new(source)).unwrap();

            assert_eq!(output, "");
            assert_eq!(remaining.byte_position(), 0);
            assert_eq!(remaining.peek(), source.chars().next());
        }
    }
}

#[test]
fn repeating_empty_tag_reports_lack_of_progress() {
    let error = repeat(tag(""), ..).parse(ParserInput::new("text")).unwrap_err();

    assert_eq!(error.byte_position(), 0);
    assert!(error.to_string().contains("does not advance"));
}
