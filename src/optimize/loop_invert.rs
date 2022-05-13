use crate::data::ast;

pub fn invert_loops(program: &mut ast::Program) {
    for function in &mut program.functions {
        invert(&mut function.statements);
    }
}

fn invert(statement: &mut ast::Statement) {
    let (condition, r#while, span) = match statement {
        ast::Statement::Assignment(_, _, _)
        | ast::Statement::Call(_)
        | ast::Statement::Initialization(_, _, _)
        | ast::Statement::Declaration(_, _)
        | ast::Statement::Return(_, _) => return,
        ast::Statement::Sequence(statements, _) => {
            for statement in statements {
                invert(statement);
            }
            return;
        }
        ast::Statement::If(_, r#if, None, _) => return invert(r#if),
        ast::Statement::If(_, r#if, Some(r#else), _) => {
            invert(r#if);
            invert(r#else);
            return;
        }
        ast::Statement::While(ast::Do::Yes, _, statement, _) => {
            invert(statement);
            return;
        }
        ast::Statement::While(ast::Do::No, condition, statement, _) if effectful(condition) => {
            invert(statement);
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
        ast::Expression::String(_, _)
        | ast::Expression::Array(_, _)
        | ast::Expression::Index(_, _, _)
        | ast::Expression::Call(_) => true,
        ast::Expression::Binary(binary, left, right, _) => match binary.get() {
            ast::Binary::Div | ast::Binary::Mod | ast::Binary::Cat => true,
            ast::Binary::Mul
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
