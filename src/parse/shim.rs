use std::str::FromStr;

use crate::data::ast;
use crate::error;
use crate::parse;
use crate::util::span;
use crate::util::symbol;

#[derive(Clone, Debug)]
pub enum PreExp {
    /// Boolean literal
    Bool(bool, span::Span),

    /// Char literal
    Chr(char, span::Span),

    /// String literal
    Str(String, span::Span),

    /// Integer literal
    Int(String, span::Span),

    /// Variable
    Var(symbol::Symbol, span::Span),

    /// Array literal
    Arr(Vec<PreExp>, span::Span),

    /// Binary operation
    Bin(ast::Bin, Box<PreExp>, Box<PreExp>, span::Span),

    /// Unary operation
    Uno(ast::Uno, Box<PreExp>, span::Span),

    /// Array index
    Idx(Box<PreExp>, Box<PreExp>, span::Span),

    /// Function call
    Call(ast::Call),
}

impl PreExp {
    pub fn span_mut(&mut self) -> &mut span::Span {
        match self {
            PreExp::Bool(_, span)
            | PreExp::Chr(_, span)
            | PreExp::Str(_, span)
            | PreExp::Int(_, span)
            | PreExp::Var(_, span)
            | PreExp::Arr(_, span)
            | PreExp::Bin(_, _, _, span)
            | PreExp::Uno(_, _, span)
            | PreExp::Idx(_, _, span)
            | PreExp::Call(ast::Call { span, .. }) => span,
        }
    }
    pub fn into_exp(self) -> Result<ast::Exp, error::Error> {
        match self {
            PreExp::Bool(b, span) => Ok(ast::Exp::Bool(b, span)),
            PreExp::Chr(c, span) => Ok(ast::Exp::Chr(c, span)),
            PreExp::Str(s, span) => Ok(ast::Exp::Str(s, span)),
            PreExp::Var(name, span) => Ok(ast::Exp::Var(name, span)),
            PreExp::Uno(ast::Uno::Neg, box PreExp::Int(mut n, _), span) => {
                n.insert(0, '-');
                i64::from_str(&n)
                    .map_err(|_| parse::Error::Integer(span).into())
                    .map(|n| ast::Exp::Int(n, span))
            }
            PreExp::Int(n, span) => i64::from_str(&n)
                .map_err(|_| parse::Error::Integer(span).into())
                .map(|n| ast::Exp::Int(n, span)),
            PreExp::Arr(exps, span) => {
                let exps = exps
                    .into_iter()
                    .map(PreExp::into_exp)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ast::Exp::Arr(exps, span))
            }
            PreExp::Bin(bin, lhs, rhs, span) => {
                let lhs = (*lhs).into_exp()?;
                let rhs = (*rhs).into_exp()?;
                Ok(ast::Exp::Bin(bin, Box::new(lhs), Box::new(rhs), span))
            }
            PreExp::Uno(uno, exp, span) => match (*exp).into_exp()? {
                ast::Exp::Int(n, _) if n == std::i64::MIN => {
                    Err(parse::Error::Integer(span).into())
                }
                ast::Exp::Int(n, _) => Ok(ast::Exp::Int(-n, span)),
                exp => Ok(ast::Exp::Uno(uno, Box::new(exp), span)),
            },
            PreExp::Idx(arr, idx, span) => {
                let arr = (*arr).into_exp()?;
                let idx = (*idx).into_exp()?;
                Ok(ast::Exp::Idx(Box::new(arr), Box::new(idx), span))
            }
            PreExp::Call(call) => Ok(ast::Exp::Call(call)),
        }
    }
}
