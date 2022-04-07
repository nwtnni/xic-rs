use std::str::FromStr;

use crate::data::ast;
use crate::error;
use crate::parse;
use crate::util::span;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub enum Expression {
    /// Boolean literal
    Boolean(bool, span::Span),

    /// Char literal
    Character(char, span::Span),

    /// String literal
    String(String, span::Span),

    /// Integer literal
    Integer(String, span::Span),

    /// Variable
    Variable(symbol::Symbol, span::Span),

    /// Array literal
    Array(Vec<Expression>, span::Span),

    /// Binary operation
    Binary(ast::Binary, Box<Expression>, Box<Expression>, span::Span),

    /// Unary operation
    Unary(ast::Unary, Box<Expression>, span::Span),

    /// Array index
    Index(Box<Expression>, Box<Expression>, span::Span),

    /// Function call
    Call(ast::Call),
}

impl Expression {
    pub fn span_mut(&mut self) -> &mut span::Span {
        match self {
            Expression::Boolean(_, span)
            | Expression::Character(_, span)
            | Expression::String(_, span)
            | Expression::Integer(_, span)
            | Expression::Variable(_, span)
            | Expression::Array(_, span)
            | Expression::Binary(_, _, _, span)
            | Expression::Unary(_, _, span)
            | Expression::Index(_, _, span)
            | Expression::Call(ast::Call { span, .. }) => span,
        }
    }

    pub fn into_expression(self) -> Result<ast::Expression, error::Error> {
        match self {
            Expression::Boolean(b, span) => Ok(ast::Expression::Boolean(b, span)),
            Expression::Character(c, span) => Ok(ast::Expression::Character(c, span)),
            Expression::String(s, span) => Ok(ast::Expression::String(s, span)),
            Expression::Variable(name, span) => Ok(ast::Expression::Variable(name, span)),
            Expression::Integer(n, span) => i64::from_str(&n)
                .map_err(|_| parse::Error::Integer(span).into())
                .map(|n| ast::Expression::Integer(n, span)),
            Expression::Array(exps, span) => {
                let exps = exps
                    .into_iter()
                    .map(Expression::into_expression)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ast::Expression::Array(exps, span))
            }
            Expression::Binary(bin, lhs, rhs, span) => {
                let lhs = (*lhs).into_expression()?;
                let rhs = (*rhs).into_expression()?;
                Ok(ast::Expression::Binary(
                    bin,
                    Box::new(lhs),
                    Box::new(rhs),
                    span,
                ))
            }
            // https://doc.rust-lang.org/beta/unstable-book/language-features/box-patterns.html
            Expression::Unary(ast::Unary::Neg, exp, span)
                if matches!(&*exp, Expression::Integer(_, _)) =>
            {
                let mut n = match *exp {
                    Expression::Integer(n, _) => n,
                    _ => unreachable!(),
                };

                n.insert(0, '-');
                i64::from_str(&n)
                    .map_err(|_| parse::Error::Integer(span).into())
                    .map(|n| ast::Expression::Integer(n, span))
            }
            Expression::Unary(uno, exp, span) => match (*exp).into_expression()? {
                ast::Expression::Integer(n, _) if n == std::i64::MIN => {
                    Err(parse::Error::Integer(span).into())
                }
                ast::Expression::Integer(n, _) => Ok(ast::Expression::Integer(-n, span)),
                exp => Ok(ast::Expression::Unary(uno, Box::new(exp), span)),
            },
            Expression::Index(arr, idx, span) => {
                let arr = (*arr).into_expression()?;
                let idx = (*idx).into_expression()?;
                Ok(ast::Expression::Index(Box::new(arr), Box::new(idx), span))
            }
            Expression::Call(call) => Ok(ast::Expression::Call(call)),
        }
    }
}
