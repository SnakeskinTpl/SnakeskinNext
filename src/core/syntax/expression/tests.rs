use crate::parsec::prelude::*;

use super::*;

fn complete(source: &str) -> Result<Expression<'_>, Box<dyn crate::parsec::ParseError>> {
    let ((expression, ()), _) = seq(expression(), ltrim(input_end()))
        .parse(ParserInput::new(source))?;

    Ok(expression)
}

fn shape(expression: &Expression<'_>) -> String {
    match &expression.kind {
        ExpressionKind::Identifier(name) => name.to_string(),
        ExpressionKind::Literal(value) => format!("{value:?}"),
        ExpressionKind::Group(inner) => format!("Group({})", shape(inner)),
        ExpressionKind::Cast { target, operand } => {
            format!("Cast({:?}, {})", target.kind, shape(operand))
        }
        ExpressionKind::Field { object, name, .. } => format!("Field({}, {name})", shape(object)),
        ExpressionKind::Call { callee, arguments, .. } => format!("Call({}, [{}])", shape(callee), arguments.iter().map(shape).collect::<Vec<_>>().join(", ")),
        ExpressionKind::Index { object, index, .. } => format!("Index({}, {})", shape(object), shape(index)),
        ExpressionKind::Unary { operator, operand, .. } => format!("{operator:?}({})", shape(operand)),
        ExpressionKind::Binary { operator, left, right, .. } => {
            format!("{operator:?}({}, {})", shape(left), shape(right))
        }
    }
}

#[test]
fn binary_precedence_builds_the_expected_tree_without_reordering() {
    for (source, expected) in [
        ("a + b * c", "Add(a, Multiply(b, c))"),
        ("a * b + c", "Add(Multiply(a, b), c)"),
        ("a + b <= c * d", "LessOrEqual(Add(a, b), Multiply(c, d))"),
        ("a < b && c != d", "And(Less(a, b), NotEqual(c, d))"),
        ("a || b && !c", "Or(a, And(b, Not(c)))"),
        ("a && b || c", "Or(And(a, b), c)"),
        ("(a + b) * c", "Multiply(Group(Add(a, b)), c)"),
        ("a && (b || c)", "And(a, Group(Or(b, c)))"),
        ("-a * b - +c / d", "Subtract(Multiply(Negate(a), b), Divide(Positive(c), d))"),
    ] {
        let parsed = complete(source).unwrap();

        assert_eq!(shape(&parsed), expected, "{source:?}");
    }
}

#[test]
fn arithmetic_and_logical_chains_associate_left() {
    for (source, expected) in [
        ("a - b - c", "Subtract(Subtract(a, b), c)"),
        ("a + b - c", "Subtract(Add(a, b), c)"),
        ("a && b && c", "And(And(a, b), c)"),
        ("a || b || c", "Or(Or(a, b), c)"),
    ] {
        let parsed = complete(source).unwrap();

        assert_eq!(shape(&parsed), expected, "{source:?}");
    }
}

#[test]
fn comparison_tokens_choose_the_full_operator() {
    for (symbol, expected) in [
        ("==", BinaryOperator::Equal), ("!=", BinaryOperator::NotEqual),
        ("<", BinaryOperator::Less), ("<=", BinaryOperator::LessOrEqual),
        (">", BinaryOperator::Greater), (">=", BinaryOperator::GreaterOrEqual),
    ] {
        let source = format!("a {symbol} b");

        let parsed = complete(&source).unwrap();
        let ExpressionKind::Binary { operator, operator_span, .. } = parsed.kind else {
            panic!("Expected a comparison");
        };

        assert_eq!(operator, expected);
        assert_eq!(&source[operator_span], symbol);
    }
}

#[test]
fn unparenthesized_comparison_chains_fail_at_the_second_operator() {
    for (source, second_operator) in [
        ("a < b < c", 6),
        ("a == b != c", 7),
        ("a < b == c", 6),
        ("a >= b > c", 7),
    ] {
        let input = ParserInput::with_offset(source, 30);

        let error = expression().parse(input).unwrap_err();

        assert_eq!(error.location().failure_byte, 30 + second_operator);
        assert!(!error.is_recoverable(), "{source:?}");
    }
}

#[test]
fn explicit_comparison_grouping_is_syntactic_and_still_requires_future_type_checks() {
    let parsed = complete("(a < b) == flag").unwrap();

    assert_eq!(shape(&parsed), "Equal(Group(Less(a, b)), flag)");
}

#[test]
fn every_consumed_binary_operator_requires_a_right_operand() {
    for operator in ["+", "-", "*", "/", "%", "==", "!=", "<", "<=", ">", ">=", "&&", "||"] {
        let source = format!("a {operator} ");

        let error = complete(&source).unwrap_err();

        assert_eq!(error.location().failure_byte, source.len(), "{source:?}");
        assert!(!error.is_recoverable());
    }
}

#[test]
fn nested_binary_ranges_use_absolute_offsets_and_exclude_whitespace() {
    let source = "  \u{0436} + b <= c  ";
    let input = ParserInput::with_offset(source, 20);

    let (parsed, remaining) = expression().parse(input).unwrap();
    let ExpressionKind::Binary { operator_span, left, right, .. } = parsed.kind else {
        panic!("Expected a comparison");
    };

    assert_eq!(remaining.source_slice(operator_span), "<=");
    assert_eq!(remaining.source_slice(left.span), "\u{0436} + b");
    assert_eq!(remaining.source_slice(right.span), "c");
    assert_eq!(remaining.source_slice(parsed.span), "\u{0436} + b <= c");

    assert_eq!(remaining.peek(), Some(' '));
}

#[test]
fn multiplicative_operators_build_binary_nodes_with_original_ranges() {
    for (symbol, expected) in [("*", BinaryOperator::Multiply), ("/", BinaryOperator::Divide), ("%", BinaryOperator::Remainder)] {
        let source = format!(" \u{0436} {symbol} 2 tail");
        let input = ParserInput::with_offset(&source, 10);

        let (parsed, remaining) = expression().parse(input).unwrap();
        let ExpressionKind::Binary { operator, operator_span, left, right } = parsed.kind else {
            panic!("Expected a binary expression");
        };

        assert_eq!(parsed.span, 11..17);
        assert_eq!(operator, expected);
        assert_eq!(operator_span, 14..15);
        assert_eq!(left.span, 11..13);
        assert_eq!(left.kind, ExpressionKind::Identifier("\u{0436}"));
        assert_eq!(right.span, 16..17);

        assert_eq!(remaining.source_slice(17..10 + source.len()), " tail");
    }
}

#[test]
fn multiplication_division_and_remainder_share_priority_and_associate_left() {
    let parsed = complete("a/b*c%d").unwrap();
    let ExpressionKind::Binary { operator, left, right, .. } = parsed.kind else { panic!("Expected remainder") };

    assert_eq!(operator, BinaryOperator::Remainder);
    assert_eq!(right.kind, ExpressionKind::Identifier("d"));

    let ExpressionKind::Binary { operator, left, right, .. } = left.kind else { panic!("Expected multiplication") };

    assert_eq!(operator, BinaryOperator::Multiply);
    assert_eq!(right.kind, ExpressionKind::Identifier("c"));

    let ExpressionKind::Binary { operator, left, right, .. } = left.kind else { panic!("Expected division") };

    assert_eq!(operator, BinaryOperator::Divide);
    assert_eq!(left.kind, ExpressionKind::Identifier("a"));
    assert_eq!(right.kind, ExpressionKind::Identifier("b"));
}

#[test]
fn unary_operators_bind_more_tightly_than_multiplication() {
    let parsed = complete("-a * +b").unwrap();
    let ExpressionKind::Binary { left, right, .. } = parsed.kind else { panic!("Expected multiplication") };

    assert!(matches!(left.kind, ExpressionKind::Unary { operator: UnaryOperator::Negate, .. }));
    assert!(matches!(right.kind, ExpressionKind::Unary { operator: UnaryOperator::Positive, .. }));

    let parsed = complete("-(a * b)").unwrap();
    let ExpressionKind::Unary { operand, .. } = parsed.kind else { panic!("Expected negation") };
    let ExpressionKind::Group(inner) = operand.kind else { panic!("Expected a group") };

    assert!(matches!(inner.kind, ExpressionKind::Binary { operator: BinaryOperator::Multiply, .. }));
}

#[test]
fn parentheses_override_left_associativity() {
    let parsed = complete("a / (b / c)").unwrap();
    let ExpressionKind::Binary { left, right, .. } = parsed.kind else { panic!("Expected division") };
    let ExpressionKind::Group(inner) = right.kind else { panic!("Expected a group") };

    assert_eq!(left.kind, ExpressionKind::Identifier("a"));
    assert!(matches!(inner.kind, ExpressionKind::Binary { operator: BinaryOperator::Divide, .. }));
}

#[test]
fn consumed_binary_operators_require_a_right_operand() {
    for (source, position) in [("a *", 3), ("a / ", 4), ("a % )", 4), ("a ** b", 3)] {
        let error = complete(source).unwrap_err();

        assert_eq!(error.location().failure_byte, position, "{source:?}");
        assert!(!error.is_recoverable());
    }
}

#[test]
fn arithmetic_preserves_signed_exponent_and_stops_before_an_unknown_operator() {
    let source = "1e-3f32 * 2 | next";

    let (parsed, remaining) = expression().parse(ParserInput::new(source)).unwrap();
    let ExpressionKind::Binary { left, .. } = parsed.kind else { panic!("Expected multiplication") };
    let ExpressionKind::Literal(LiteralKind::Number(number)) = left.kind else { panic!("Expected a number") };

    assert_eq!(number.content, "1e-3");
    assert_eq!(remaining.source_slice(remaining.byte_position()..source.len()), " | next");
}

#[test]
fn numeric_prefixes_preserve_operator_and_operand_spans() {
    for (source, expected) in [("  -12 tail", UnaryOperator::Negate), ("  +12 tail", UnaryOperator::Positive)] {
        let input = ParserInput::with_offset(source, 10);

        let (parsed, remaining) = expression().parse(input).unwrap();
        let ExpressionKind::Unary { operator, operator_span, operand } = parsed.kind else {
            panic!("Expected a unary expression");
        };

        assert_eq!(parsed.span, 12..15);
        assert_eq!(operator, expected);
        assert_eq!(operator_span, 12..13);
        assert_eq!(operand.span, 13..15);

        let ExpressionKind::Literal(LiteralKind::Number(number)) = operand.kind else {
            panic!("Expected a numeric literal");
        };

        assert_eq!(number.content, "12");
        assert_eq!(remaining.peek(), Some(' '));
    }
}

#[test]
fn mixed_prefixes_nest_to_the_right_and_accept_identifier_operands() {
    let parsed = complete("-+!flag").unwrap();
    let ExpressionKind::Unary { operator, operand, .. } = parsed.kind else { panic!("Expected minus") };

    assert_eq!(operator, UnaryOperator::Negate);

    let ExpressionKind::Unary { operator, operand, .. } = operand.kind else { panic!("Expected plus") };

    assert_eq!(operator, UnaryOperator::Positive);

    let ExpressionKind::Unary { operator, operand, .. } = operand.kind else { panic!("Expected not") };

    assert_eq!(operator, UnaryOperator::Not);
    assert_eq!(operand.kind, ExpressionKind::Identifier("flag"));
}

#[test]
fn numeric_prefixes_accept_groups_and_keep_exponent_sign_inside_the_literal() {
    let parsed = complete("-(+1e-3f32)").unwrap();
    let ExpressionKind::Unary { operand, .. } = parsed.kind else { panic!("Expected minus") };
    let ExpressionKind::Group(inner) = operand.kind else { panic!("Expected a group") };
    let ExpressionKind::Unary { operator, operand, .. } = inner.kind else { panic!("Expected plus") };
    let ExpressionKind::Literal(LiteralKind::Number(number)) = operand.kind else { panic!("Expected a number") };

    assert_eq!(operator, UnaryOperator::Positive);
    assert_eq!(number.content, "1e-3");
}

#[test]
fn negative_minimum_is_preserved_for_later_type_range_validation() {
    let parsed = complete("-128i8").unwrap();
    let ExpressionKind::Unary { operator, operand, .. } = parsed.kind else { panic!("Expected minus") };
    let ExpressionKind::Literal(LiteralKind::Number(number)) = operand.kind else { panic!("Expected a number") };

    assert_eq!(operator, UnaryOperator::Negate);
    assert_eq!(number.content, "128");
    assert_eq!(number.suffix, Some(super::super::literal::NumberSuffix::Int { bits: Some("8") }));
}

#[test]
fn numeric_prefixes_require_an_operand_after_every_operator() {
    for (source, position) in [("-", 1), ("+ ", 2), ("-+", 2), ("-( )", 3)] {
        let error = complete(source).unwrap_err();

        assert_eq!(error.location().failure_byte, position, "{source:?}");
        assert!(!error.is_recoverable());
    }
}

#[test]
fn negation_preserves_operator_and_literal_ranges() {
    let input = ParserInput::with_offset("  ! true tail", 20);

    let (parsed, remaining) = expression().parse(input).unwrap();
    let ExpressionKind::Unary { operator, operator_span, operand } = parsed.kind else {
        panic!("Expected a unary expression");
    };

    assert_eq!(parsed.span, 22..28);
    assert_eq!(operator, UnaryOperator::Not);
    assert_eq!(operator_span, 22..23);
    assert_eq!(operand.span, 24..28);
    assert_eq!(operand.kind, ExpressionKind::Literal(LiteralKind::Bool(true)));

    assert_eq!(remaining.source_slice(28..33), " tail");
}

#[test]
fn repeated_negations_nest_to_the_right() {
    let parsed = complete("!!flag").unwrap();
    let ExpressionKind::Unary { operator_span, operand, .. } = parsed.kind else {
        panic!("Expected an outer negation");
    };

    assert_eq!(operator_span, 0..1);
    assert_eq!(operand.span, 1..6);

    let ExpressionKind::Unary { operator_span, operand, .. } = operand.kind else {
        panic!("Expected an inner negation");
    };

    assert_eq!(operator_span, 1..2);
    assert_eq!(operand.span, 2..6);
    assert_eq!(operand.kind, ExpressionKind::Identifier("flag"));
}

#[test]
fn grouping_retains_parentheses_and_inner_ranges() {
    let parsed = complete("!( !flag )").unwrap();
    let ExpressionKind::Unary { operand, .. } = parsed.kind else { panic!("Expected negation") };

    assert_eq!(operand.span, 1..10);

    let ExpressionKind::Group(inner) = operand.kind else { panic!("Expected a group") };

    assert_eq!(inner.span, 3..8);
    assert!(matches!(inner.kind, ExpressionKind::Unary { .. }));
}

#[test]
fn literal_prefixes_do_not_hide_names_and_unicode_offsets_remain_in_bytes() {
    for name in ["trueValue", "false_", "nullable", "\u{0436}"] {
        let source = format!("!{name}");

        let parsed = complete(&source).unwrap();
        let ExpressionKind::Unary { operand, .. } = parsed.kind else { panic!("Expected negation") };

        assert_eq!(operand.kind, ExpressionKind::Identifier(name));
        assert_eq!(operand.span, 1..source.len());
    }
}

#[test]
fn missing_operands_and_unclosed_groups_are_committed_errors() {
    for (source, position) in [("!", 1), ("! ", 2), ("!(true", 6), ("!()", 2), ("(!)", 2)] {
        let error = complete(source).unwrap_err();

        assert!(!error.is_recoverable(), "{source:?}");
        assert_eq!(error.location().failure_byte, position, "{source:?}");
    }
}

#[test]
fn malformed_literals_inside_negation_keep_their_own_diagnostics() {
    for (source, position) in [("!1.3u32", 4), ("!\"\\q\"", 2)] {
        let error = complete(source).unwrap_err();

        assert_eq!(error.location().failure_byte, position);
        assert!(!error.is_recoverable());
    }
}

#[test]
fn syntax_does_not_perform_boolean_type_checking() {
    let parsed = complete("!1").unwrap();
    let ExpressionKind::Unary { operand, .. } = parsed.kind else { panic!("Expected negation") };

    assert!(matches!(operand.kind, ExpressionKind::Literal(LiteralKind::Number(_))));
}

#[test]
fn grammar_leaves_unknown_operators_for_future_expression_layers() {
    let (parsed, remaining) = expression().parse(ParserInput::new("!flag | other")).unwrap();

    assert_eq!(parsed.span, 0..5);
    assert_eq!(remaining.source_slice(5..13), " | other");
}

#[test]
fn postfix_chains_bind_more_tightly_than_unary_and_binary_operations() {
    for (source, expected) in [
        ("!user.active", "Not(Field(user, active))"),
        ("-check(a, b)[index].value + other", "Add(Negate(Field(Index(Call(check, [a, b]), index), value)), other)"),
        ("factory()()", "Call(Call(factory, []), [])"),
        ("tuple.0", "Field(tuple, 0)"),
        ("(a + b).method()", "Call(Field(Group(Add(a, b)), method), [])"),
    ] {
        let parsed = complete(source).unwrap();

        assert_eq!(shape(&parsed), expected, "{source:?}");
    }
}

#[test]
fn numeric_method_access_does_not_start_a_fraction() {
    let parsed = complete("1.method()").unwrap();
    let ExpressionKind::Call { callee, arguments, arguments_span } = parsed.kind else { panic!("Expected call") };
    let ExpressionKind::Field { object, name, name_span } = callee.kind else { panic!("Expected field") };

    assert_eq!(name, "method");
    assert_eq!(name_span, 2..8);
    assert_eq!(object.span, 0..1);
    assert_eq!(arguments_span, 8..10);
    assert!(arguments.is_empty());
}

#[test]
fn postfix_ranges_include_delimiters_and_preserve_absolute_offsets() {
    let source = "  obj .field (arg)[idx] ";
    let (parsed, remaining) = expression().parse(ParserInput::with_offset(source, 10)).unwrap();
    let ExpressionKind::Index { object, index, brackets_span } = parsed.kind else { panic!("Expected index") };
    let ExpressionKind::Call { arguments_span, .. } = object.kind else { panic!("Expected call") };

    assert_eq!(parsed.span, 12..33);
    assert_eq!(brackets_span, 28..33);
    assert_eq!(index.span, 29..32);
    assert_eq!(arguments_span, 23..28);
    assert_eq!(remaining.byte_position(), 33);
}

#[test]
fn incomplete_postfix_operations_are_committed_errors() {
    for source in ["obj.", "f(", "f(a,)", "f(,a)", "f(a b)", "items[]", "items[index", "items[a, b]"] {
        let error = complete(source).unwrap_err();

        assert!(!error.is_recoverable(), "{source:?}");
    }
}

#[test]
fn range_prefix_is_not_consumed_as_field_access() {
    let (parsed, remaining) = expression().parse(ParserInput::new("a..b")).unwrap();

    assert_eq!(parsed.span, 0..1);
    assert_eq!(remaining.byte_position(), 1);
}

#[test]
fn unicode_fields_keep_byte_ranges_and_index_expressions_keep_precedence() {
    let source = "obj.\u{0438}()[a + b * c]";

    let parsed = complete(source).unwrap();
    let ExpressionKind::Index { object, index, .. } = parsed.kind else { panic!("Expected index") };
    let ExpressionKind::Call { callee, .. } = object.kind else { panic!("Expected call") };
    let ExpressionKind::Field { name_span, .. } = callee.kind else { panic!("Expected field") };

    assert_eq!(name_span, 4..6);
    assert_eq!(parsed.span, 0..source.len());
    assert_eq!(shape(&index), "Add(a, Multiply(b, c))");
}

#[test]
fn casts_bind_like_unary_operators_and_preserve_postfix_chains() {
    for (source, expected) in [
        ("<Int>a + b", "Add(Cast(Named(\"Int\"), a), b)"),
        ("<Int>f().value", "Cast(Named(\"Int\"), Field(Call(f, []), value))"),
        ("<Int>(a + b)", "Cast(Named(\"Int\"), Group(Add(a, b)))"),
        ("!<Bool>flag", "Not(Cast(Named(\"Bool\"), flag))"),
        ("<Bool>!flag", "Cast(Named(\"Bool\"), Not(flag))"),
        ("<Int><Other>value", "Cast(Named(\"Int\"), Cast(Named(\"Other\"), value))"),
        ("a < <Int>b", "Less(a, Cast(Named(\"Int\"), b))"),
    ] {
        let parsed = complete(source).unwrap();

        assert_eq!(shape(&parsed), expected, "{source:?}");
    }
}

#[test]
fn casts_accept_nested_type_and_value_arguments() {
    use super::super::type_syntax::{TypeSyntaxKind, TypeArgumentSyntax};

    for source in ["<Int<64>>1", "<Opt<Vec<Str>>>value", "<Array<Int, 3>>items"] {
        let parsed = complete(source).unwrap();
        let ExpressionKind::Cast { target, operand } = parsed.kind else { panic!("Expected cast") };
        let TypeSyntaxKind::Applied { arguments, .. } = target.kind else { panic!("Expected applied type") };

        assert!(!arguments.is_empty());
        assert_eq!(parsed.span, 0..source.len());
        assert_eq!(operand.span.end, source.len());

        if source == "<Int<64>>1" {
            assert!(matches!(arguments[0], TypeArgumentSyntax::Value(_)));
            assert!(matches!(operand.kind, ExpressionKind::Literal(LiteralKind::Number(_))));
        }
    }
}

#[test]
fn cast_ranges_exclude_whitespace_and_keep_absolute_offsets() {
    use super::super::type_syntax::TypeSyntaxKind;

    let source = "  < Bool > flag ";

    let (parsed, remaining) = expression().parse(ParserInput::with_offset(source, 20)).unwrap();
    let ExpressionKind::Cast { target, operand } = parsed.kind else { panic!("Expected cast") };

    assert_eq!(parsed.span, 22..35);
    assert_eq!(target.span, 24..28);
    assert_eq!(target.kind, TypeSyntaxKind::Named("Bool"));
    assert_eq!(operand.span, 31..35);
    assert_eq!(remaining.byte_position(), 35);
}

#[test]
fn incomplete_casts_commit_after_the_opening_bracket() {
    for (source, position) in [
        ("<", 1),
        ("<>flag", 1),
        ("<bool>flag", 1),
        ("<Bool", 5),
        ("<Bool>", 6),
        ("<Bool> ", 7),
        ("<Opt<Bool>value", 10),
    ] {
        let error = complete(source).unwrap_err();

        assert!(!error.is_recoverable(), "{source:?}");
        assert_eq!(error.location().failure_byte, position, "{source:?}");
    }
}
