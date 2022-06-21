use std::path::Path;

use crate::data::ast;
use crate::util;

// FIXME: operate on typed AST
pub fn invert_ast(path: &Path, program: &mut ast::Program<()>) {
    log::info!(
        "[{}] Inverting loops in {}...",
        std::any::type_name::<ast::Program<()>>(),
        path.display()
    );
    util::time!(
        "[{}] Done inverting loops in {}",
        std::any::type_name::<ast::Program<()>>(),
        path.display()
    );

    let mut inverter = Inverter(0);
    program.accept_mut(&mut inverter);
    log::debug!("Inverted {} loops!", inverter.0);
}

struct Inverter(usize);

impl ast::VisitorMut<()> for Inverter {
    fn visit_statement(&mut self, statement: &mut ast::Statement<()>) {
        if let ast::Statement::While(ast::Do::No, condition, r#while, span) = statement {
            if !effectful(condition) {
                log::trace!("Inverted loop at {}", span);
                self.0 += 1;

                *statement = ast::Statement::If(
                    Box::new(condition.negate_logical()),
                    Box::new(ast::Statement::Sequence(Vec::new(), *span)),
                    Some(Box::new(ast::Statement::While(
                        ast::Do::Yes,
                        condition.clone(),
                        r#while.clone(),
                        *span,
                    ))),
                    *span,
                );
            }
        }
    }
}

fn effectful(expression: &ast::Expression<()>) -> bool {
    match expression {
        ast::Expression::Boolean(_, _)
        | ast::Expression::Character(_, _)
        | ast::Expression::Integer(_, _)
        | ast::Expression::Variable(_, _)
        | ast::Expression::This(_, _)
        | ast::Expression::Super(_, _)
        | ast::Expression::Null(_, _) => false,

        // Recomputing these shouldn't be observable, but it _is_ inefficient.
        ast::Expression::String(_, _)
        | ast::Expression::Array(_, _, _)
        | ast::Expression::New(_, _) => true,

        // Note: it's safe to hoist an index even if it may
        // crash, since it's evaluated at least once whether
        // or not we invert the loop. There can be no other
        // effects other than crashing.
        ast::Expression::Index(array, index, _, _) => effectful(array) || effectful(index),
        ast::Expression::Length(array, _) => effectful(array),
        ast::Expression::Dot(_, expression, _, _, _) => effectful(expression),

        ast::Expression::Call(_) => true,
        ast::Expression::Binary(binary, left, right, _, _) => match binary.get() {
            // Avoid recomputing array concatenation, which is expensive.
            ast::Binary::Cat => true,

            // Note: division and modulo are safe for the same
            // reason as indexing is safe.
            ast::Binary::Div
            | ast::Binary::Mod
            | ast::Binary::Mul
            | ast::Binary::Hul
            | ast::Binary::Add
            | ast::Binary::Sub
            | ast::Binary::Lt
            | ast::Binary::Le
            | ast::Binary::Ge
            | ast::Binary::Gt
            | ast::Binary::Eq
            | ast::Binary::Ne
            | ast::Binary::And
            | ast::Binary::Or => effectful(left) || effectful(right),
        },
        ast::Expression::Unary(ast::Unary::Neg | ast::Unary::Not, expression, _, _) => {
            effectful(expression)
        }
    }
}
