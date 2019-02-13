use std::str::FromStr;

use crate::ast;
use crate::error;
use crate::parse;
use crate::span;
use crate::symbol;

#[derive(Clone, Debug)]
pub enum Prexp {
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
    Arr(Vec<Prexp>, span::Span),

    /// Binary operation
    Bin(ast::Bin, Box<Prexp>, Box<Prexp>, span::Span),

    /// Unary operation
    Uno(ast::Uno, Box<Prexp>, span::Span),

    /// Array index
    Idx(Box<Prexp>, Box<Prexp>, span::Span),

    /// Function call
    Call(symbol::Symbol, Vec<Prexp>, span::Span),
}

impl Prexp {
    pub fn into(self) -> Result<ast::Exp, error::Error> {
        match self {
        | Prexp::Bool(b, span)   => Ok(ast::Exp::Bool(b, span)),
        | Prexp::Chr(c, span)    => Ok(ast::Exp::Chr(c, span)),
        | Prexp::Str(s, span)    => Ok(ast::Exp::Str(s, span)),
        | Prexp::Var(name, span) => Ok(ast::Exp::Var(name, span)),
        | Prexp::Uno(ast::Uno::Neg, box Prexp::Int(mut n, _), span) => {
            n.insert(0, '-'); 
            i64::from_str(&n)
                .map_err(|_| parse::Error::Integer(span).into())
                .map(|n| ast::Exp::Int(n, span))
        }
        | Prexp::Int(n, span) => {
            i64::from_str(&n)
                .map_err(|_| parse::Error::Integer(span).into())
                .map(|n| ast::Exp::Int(n, span))
        }
        | Prexp::Arr(exps, span) => {
            let exps = exps.into_iter()
                .map(Prexp::into)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ast::Exp::Arr(exps, span))
        }
        | Prexp::Bin(bin, lhs, rhs, span) => {
            let lhs = (*lhs).into()?; 
            let rhs = (*rhs).into()?; 
            Ok(ast::Exp::Bin(bin, Box::new(lhs), Box::new(rhs), span))
        }
        | Prexp::Uno(uno, exp, span) => {
            match (*exp).into()? {
            | ast::Exp::Int(n, _) if n == std::i64::MIN => {
                Err(parse::Error::Integer(span).into())
            }
            | ast::Exp::Int(n, _) => {
                Ok(ast::Exp::Int(-n, span))
            }
            | exp => {
                Ok(ast::Exp::Uno(uno, Box::new(exp), span))
            }
            }
        }
        | Prexp::Idx(arr, idx, span) => {
            let arr = (*arr).into()?;
            let idx = (*idx).into()?;
            Ok(ast::Exp::Idx(Box::new(arr), Box::new(idx), span))
        }
        | Prexp::Call(name, args, span) => {
            let args = args.into_iter()
                .map(Prexp::into)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ast::Exp::Call(name, args, span))
        }
        }
    }
}
