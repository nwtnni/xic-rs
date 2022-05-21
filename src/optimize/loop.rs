use crate::data::ast;

pub fn invert_ast(program: &mut ast::Program) {
    for item in &mut program.items {
        match item {
            ast::Item::Global(_) => (),
            ast::Item::Class(class) => {
                for item in &mut class.items {
                    match item {
                        ast::ClassItem::Field(_) => (),
                        ast::ClassItem::Method(method) => {
                            invert_statement(&mut method.statements);
                        }
                    }
                }
            }
            ast::Item::Function(function) => {
                invert_statement(&mut function.statements);
            }
        }
    }
}

fn invert_statement(statement: &mut ast::Statement) {
    let (condition, r#while, span) = match statement {
        ast::Statement::Assignment(_, _, _)
        | ast::Statement::Call(_)
        | ast::Statement::Initialization(_)
        | ast::Statement::Declaration(_, _)
        | ast::Statement::Return(_, _)
        | ast::Statement::Break(_) => return,
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
        | ast::Expression::Variable(_)
        | ast::Expression::This(_)
        | ast::Expression::Null(_) => false,

        // Recomputing these shouldn't be observable, but it _is_ inefficient.
        ast::Expression::String(_, _)
        | ast::Expression::Array(_, _)
        | ast::Expression::New(_, _) => true,

        // Note: it's safe to hoist an index even if it may
        // crash, since it's evaluated at least once whether
        // or not we invert the loop. There can be no other
        // effects other than crashing.
        ast::Expression::Index(array, index, _) => effectful(array) || effectful(index),
        ast::Expression::Length(array, _) => effectful(array),
        ast::Expression::Dot(expression, _, _) => effectful(expression),

        ast::Expression::Call(_) => true,
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
