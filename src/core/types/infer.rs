use crate::core::syntax::expression::{BinaryOperator, Expression, ExpressionKind, UnaryOperator};
use crate::core::syntax::literal::LiteralKind;

use super::literal::literal_type;
use super::resolve::resolve;
use super::{ValueInfo, Type, TypeEnvironment, TypeError, TypeErrorKind};

/// Infers types and preserves literal data without evaluating expressions.
pub(crate) fn infer<'a>(expression: &Expression<'a>, environment: &TypeEnvironment<'_>) -> Result<ValueInfo<'a>, TypeError> {
    infer_expected(expression, environment, None)
}

fn infer_expected<'a>(expression: &Expression<'a>, environment: &TypeEnvironment<'_>, expected: Option<Type>) -> Result<ValueInfo<'a>, TypeError> {
    let unsupported = |construct| TypeError {
        span: expression.span.clone(),
        kind: TypeErrorKind::Unsupported { construct },
    };

    match &expression.kind {
        ExpressionKind::Literal(value) => Ok(ValueInfo {
            ty: literal_type(value, expected, expression.span.clone())?,
            literal: Some(value.clone()),
        }),

        ExpressionKind::Identifier(name) => environment.get(name).copied()
            .map(ValueInfo::computed)
            .ok_or_else(|| TypeError {
                span: expression.span.clone(),
                kind: TypeErrorKind::UnknownIdentifier { name: (*name).into() },
            }),

        ExpressionKind::Group(inner) => infer_expected(inner, environment, expected),
        ExpressionKind::Unary { operator, operator_span, operand } => {
            let operand = infer_expected(operand, environment, expected)?.ty;

            match (operator, operand) {
                (UnaryOperator::Not, Type::Bool) => Ok(ValueInfo::computed(Type::Bool)),
                (UnaryOperator::Positive, ty) if ty.is_numeric() => Ok(ValueInfo::computed(ty)),
                (UnaryOperator::Negate, ty @ (Type::Int { .. } | Type::Float { .. })) => Ok(ValueInfo::computed(ty)),

                _ => Err(TypeError {
                    span: operator_span.clone(),
                    kind: TypeErrorKind::InvalidUnary { operator: *operator, operand },
                }),
            }
        }

        ExpressionKind::Binary { operator, operator_span, left, right } => {
            // Read explicit operand types before applying defaults, in either order.
            let context = hint(left, environment).or_else(|| hint(right, environment))
                .or(expected.filter(|_| !is_comparison(*operator)));

            let left = infer_expected(left, environment, context)?.ty;
            let right = infer_expected(right, environment, context)?.ty;
            let matching = left == right;

            let ty = match operator {
                BinaryOperator::And | BinaryOperator::Or if left == Type::Bool && right == Type::Bool => Some(Type::Bool),
                BinaryOperator::Equal | BinaryOperator::NotEqual if matching => Some(Type::Bool),

                BinaryOperator::Less | BinaryOperator::LessOrEqual | BinaryOperator::Greater | BinaryOperator::GreaterOrEqual
                    if matching && left.is_numeric() => Some(Type::Bool),

                BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Remainder
                    if matching && left.is_numeric() => Some(left),

                _ => None,
            };

            ty.map(ValueInfo::computed).ok_or_else(|| TypeError {
                span: operator_span.clone(),
                kind: TypeErrorKind::InvalidBinary { operator: *operator, left, right },
            })
        }

        ExpressionKind::Cast { target, operand } => {
            let expected = resolve(target)?;
            let value_info = infer_expected(operand, environment, Some(expected))?;

            if value_info.ty == expected {
                return Ok(value_info);
            }

            // TODO: Validate non-identity casts and represent conversions in typed IR.
            Err(TypeError {
                span: expression.span.clone(),
                kind: TypeErrorKind::MismatchedType { expected, found: value_info.ty },
            })
        }

        // TODO: Infer postfix operations from field, function and container types.
        ExpressionKind::Field { .. } => Err(unsupported("field access")),
        ExpressionKind::Call { .. } => Err(unsupported("calls")),
        ExpressionKind::Index { .. } => Err(unsupported("indexing")),
    }
}

fn is_comparison(operator: BinaryOperator) -> bool {
    matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual | BinaryOperator::Less |
        BinaryOperator::LessOrEqual | BinaryOperator::Greater | BinaryOperator::GreaterOrEqual)
}

/// Unsuffixed numbers do not contribute a default until available context is known.
fn hint(expression: &Expression<'_>, environment: &TypeEnvironment<'_>) -> Option<Type> {
    match &expression.kind {
        ExpressionKind::Identifier(name) => environment.get(name).copied(),
        ExpressionKind::Literal(value @ LiteralKind::Number(number)) if number.suffix.is_some() => {
            literal_type(value, None, expression.span.clone()).ok()
        }

        ExpressionKind::Group(inner) => hint(inner, environment),
        ExpressionKind::Unary { operator: UnaryOperator::Positive | UnaryOperator::Negate, operand, .. } => hint(operand, environment),
        ExpressionKind::Cast { target, .. } => resolve(target).ok(),
        ExpressionKind::Binary { operator, left, right, .. } if !is_comparison(*operator) => {
            hint(left, environment).or_else(|| hint(right, environment))
        }

        _ => None,
    }
}
