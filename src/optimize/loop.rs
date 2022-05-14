use crate::data::ast;
use crate::data::symbol;

pub fn invert_ast(program: &mut ast::Program) {
    for function in &mut program.functions {
        invert_statement(&mut function.statements);
    }
}

fn invert_statement(statement: &mut ast::Statement) {
    let (condition, r#while, span) = match statement {
        ast::Statement::Assignment(_, _, _)
        | ast::Statement::Call(_)
        | ast::Statement::Initialization(_, _, _)
        | ast::Statement::Declaration(_, _)
        | ast::Statement::Return(_, _) => return,
        ast::Statement::Sequence(statements, _) => {
            for statement in statements {
                invert_statement(statement);
            }
            return;
        }
        ast::Statement::If(_, r#if, None, _) => return invert_statement(r#if),
        ast::Statement::If(_, r#if, Some(r#else), _) => {
            invert_statement(r#if);
            invert_statement(r#else);
            return;
        }
        ast::Statement::While(ast::Do::Yes, _, statement, _) => {
            invert_statement(statement);
            return;
        }
        ast::Statement::While(ast::Do::No, condition, statement, _) if effectful(condition) => {
            invert_statement(statement);
            return;
        }
        ast::Statement::While(ast::Do::No, condition, statement, span) => {
            (condition.clone(), statement.clone(), *span)
        }
    };

    *statement = ast::Statement::If(
        condition.clone(),
        Box::new(ast::Statement::While(
            ast::Do::Yes,
            condition,
            r#while,
            span,
        )),
        None,
        span,
    );
}

fn effectful(expression: &ast::Expression) -> bool {
    match expression {
        ast::Expression::Boolean(_, _)
        | ast::Expression::Character(_, _)
        | ast::Expression::Integer(_, _)
        | ast::Expression::Variable(_, _) => false,

        // Recomputing these shouldn't be observable, but it _is_ inefficient.
        ast::Expression::String(_, _) | ast::Expression::Array(_, _) => true,

        // Note: it's safe to hoist an index even if it may
        // crash, since it's evaluated at least once whether
        // or not we invert the loop. There can be no other
        // effects other than crashing.
        ast::Expression::Index(array, index, _) => effectful(array) || effectful(index),
        ast::Expression::Call(call) => symbol::resolve(call.name) != "length",
        ast::Expression::Binary(binary, left, right, _) => match binary.get() {
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
        ast::Expression::Unary(ast::Unary::Neg | ast::Unary::Not, expression, _) => {
            effectful(expression)
        }
    }
}