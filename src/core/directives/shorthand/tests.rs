use super::*;

#[test]
fn registers_paired_delimiters_through_the_directive_trait() {
    let source = String::from(r#"tag { opening: "<", closing: ">" }"#);
    let directive: Box<dyn Directive> = Box::new(Shorthand::new());
    let mut context = DirectiveContext::default();

    directive.begin(&mut context, ParserInput::new(&source)).unwrap();

    let binding = context.shorthands.get("<").unwrap();

    assert_eq!(binding.opening, "<");
    assert_eq!(binding.closing.as_deref(), Some(">"));
    assert_eq!(binding.target, "tag");
    assert_eq!(binding.target.as_ptr(), source.as_ptr());
    assert!(matches!(binding.opening, Cow::Borrowed(_)));
    assert_eq!(&source[binding.opening_span.clone()], "\"<\"");
    assert_eq!(binding.span, 0..source.len());
}

#[test]
fn supports_unpaired_and_escaped_multicharacter_delimiters() {
    let directive = Shorthand::new();
    let mut context = DirectiveContext::default();

    directive.begin(&mut context, ParserInput::new(r#"tag_2 {opening: "\u003c<<"}"#)).unwrap();

    let binding = context.shorthands.get("<<<").unwrap();

    assert_eq!(binding.target, "tag_2");
    assert_eq!(binding.closing, None);
    assert!(matches!(binding.opening, Cow::Owned(_)));
}

#[test]
fn invalid_configuration_does_not_modify_the_environment() {
    let directive = Shorthand::new();
    let mut context = DirectiveContext::default();
    directive.begin(&mut context, ParserInput::new(r#"tag {opening: "<"}"#)).unwrap();

    for source in ["", "tag", "tag {}", "tag []", "tag {opening: 1}",
        r#"tag {opening: ""}"#, r#"tag {opening: "[", closing: ""}"#,
        r#"tag {opening: "[", closing: false}"#, r#"tag {opening: "[", extra: 1}"#,
        r#"tag {opening: "[", opening: "]"}"#, r#"tag {opening: "["} extra"#,
        r#"tag {opening: "[", closing: "]", closing: "}"}"#] {
        let result = directive.begin(&mut context, ParserInput::new(source));

        assert!(result.is_err(), "Expected invalid declaration: {source:?}");
        assert_eq!(context.shorthands.len(), 1);
        assert_eq!(context.shorthands.get("<").unwrap().target, "tag");
    }
}

#[test]
fn duplicate_prefix_reports_original_byte_ranges_and_preserves_first_binding() {
    let first = r#"tag {opening: "<"}"#;
    let second = r#"ignored output {opening: "\u003c"}"#;
    let directive = Shorthand::new();
    let mut context = DirectiveContext::default();
    directive.begin(&mut context, ParserInput::new(first)).unwrap();
    let mut input = ParserInput::new(second);
    input.set_byte_position("ignored ".len());

    let error = directive.begin(&mut context, input).unwrap_err();

    assert!(!error.is_recoverable());
    assert_eq!(error.location().start_byte, "ignored ".len());
    assert_eq!(error.location().failure_byte, second.find('"').unwrap());
    assert_eq!(error.location().found_char, Some('"'));

    let duplicate = (&*error as &dyn std::error::Error)
        .downcast_ref::<DuplicateShorthandError>().unwrap();

    assert_eq!(duplicate.previous_span, 0..first.len());
    assert_eq!(context.shorthands.len(), 1);
    assert_eq!(context.shorthands.get("<").unwrap().target, "tag");
}

#[test]
fn different_prefixes_can_target_the_same_directive() {
    let directive = Shorthand::new();
    let mut context = DirectiveContext::default();

    directive.begin(&mut context, ParserInput::new(r#"tag {opening: "<"}"#)).unwrap();
    directive.begin(&mut context, ParserInput::new(r#"tag {opening: "<<"}"#)).unwrap();

    assert_eq!(context.shorthands.len(), 2);
    assert_eq!(context.shorthands.get("<").unwrap().target, "tag");
    assert_eq!(context.shorthands.get("<<").unwrap().target, "tag");
}

#[test]
fn requires_a_separator_and_preserves_ranges_with_optional_whitespace() {
    let source = " \ttag\u{2003}{opening: \"<\"} \r\n";
    let directive = Shorthand::new();
    let mut context = DirectiveContext::default();

    directive.begin(&mut context, ParserInput::new(source)).unwrap();

    assert_eq!(context.shorthands.get("<").unwrap().target, "tag");
    assert_eq!(context.shorthands.get("<").unwrap().span, 0..source.len());

    let error = directive.begin(&mut context, ParserInput::new(r#"tag{opening: "["}"#));

    assert!(error.is_err());
    assert_eq!(context.shorthands.len(), 1);
}
