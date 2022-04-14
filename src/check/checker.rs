use crate::check::context;
use crate::check::Context;
use crate::check::Error;
use crate::check::ErrorKind;
use crate::data::ast;
use crate::data::r#type;
use crate::error;
use crate::lex;
use crate::parse;
use crate::data::symbol;

macro_rules! bail {
    ($span:expr, $kind:expr) => {
        return Err(Error::new($span, $kind).into())
    };
}

macro_rules! expected {
    ($span:expr, $expected:expr, $found:expr) => {{
        let kind = ErrorKind::Mismatch {
            expected: $expected,
            found: $found,
        };
        bail!($span, kind)
    }};
}

pub struct Checker {
    context: Context,
}

impl Checker {
    pub fn new() -> Self {
        Checker {
            context: Context::new(),
        }
    }

    pub fn check_program(
        mut self,
        directory_library: &std::path::Path,
        program: &ast::Program,
    ) -> Result<Context, error::Error> {
        for path in &program.uses {
            let path = directory_library.join(symbol::resolve(path.name).to_string() + ".ixi");
            let source = std::fs::read_to_string(path)?;
            let lexer = lex::Lexer::new(&source);
            let interface = parse::InterfaceParser::new().parse(lexer)?;
            self.load_interface(&interface)?;
        }

        for function in &program.functions {
            self.load_function(function)?;
        }

        for function in &program.functions {
            self.check_function(function)?;
        }

        Ok(self.context)
    }

    fn load_interface(&mut self, interface: &ast::Interface) -> Result<(), error::Error> {
        for signature in &interface.signatures {
            let (name, new_parameters, new_returns) = self.check_signature(signature)?;

            match self.context.get(name) {
                Some(context::Entry::Signature(old_parameters, _))
                    if new_parameters != *old_parameters =>
                {
                    bail!(signature.span, ErrorKind::NameClash)
                }
                Some(context::Entry::Signature(_, old_returns)) if new_returns != *old_returns => {
                    bail!(signature.span, ErrorKind::NameClash)
                }
                Some(context::Entry::Signature(_, _)) | None => {
                    self.context
                        .insert(name, context::Entry::Signature(new_parameters, new_returns));
                }
                Some(_) => bail!(signature.span, ErrorKind::NameClash),
            }
        }
        Ok(())
    }

    fn load_function(&mut self, function: &ast::Function) -> Result<(), error::Error> {
        let (name, new_parameters, new_returns) = self.check_signature(function)?;

        match self.context.remove(name) {
            Some(context::Entry::Signature(old_parameters, _))
                if !r#type::subtypes(&old_parameters, &new_parameters) =>
            {
                bail!(function.span, ErrorKind::SignatureMismatch)
            }
            Some(context::Entry::Signature(_, old_returns))
                if !r#type::subtypes(&new_returns, &old_returns) =>
            {
                bail!(function.span, ErrorKind::SignatureMismatch)
            }
            Some(context::Entry::Signature(_, _)) | None => {
                self.context
                    .insert(name, context::Entry::Function(new_parameters, new_returns));
            }
            Some(_) => bail!(function.span, ErrorKind::NameClash),
        }

        Ok(())
    }

    fn check_signature<C: ast::Callable>(
        &self,
        signature: &C,
    ) -> Result<
        (
            symbol::Symbol,
            Vec<r#type::Expression>,
            Vec<r#type::Expression>,
        ),
        error::Error,
    > {
        let parameters = signature
            .parameters()
            .iter()
            .map(|declaration| &declaration.r#type)
            .map(|r#type| self.check_type(r#type))
            .collect::<Result<Vec<_>, _>>()?;

        let returns = signature
            .returns()
            .iter()
            .map(|r#type| self.check_type(r#type))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((signature.name(), parameters, returns))
    }

    fn check_type(&self, r#type: &ast::Type) -> Result<r#type::Expression, error::Error> {
        match r#type {
            ast::Type::Bool(_) => Ok(r#type::Expression::Boolean),
            ast::Type::Int(_) => Ok(r#type::Expression::Integer),
            ast::Type::Array(r#type, None, _) => self
                .check_type(r#type)
                .map(Box::new)
                .map(r#type::Expression::Array),
            ast::Type::Array(r#type, Some(len), _) => {
                let r#type = self.check_type(r#type)?;
                match self.check_expression(len)? {
                    r#type::Expression::Integer => Ok(r#type::Expression::Array(Box::new(r#type))),
                    r#type => expected!(len.span(), r#type::Expression::Integer, r#type),
                }
            }
        }
    }

    fn check_function(&mut self, function: &ast::Function) -> Result<(), error::Error> {
        let returns = match self.context.get(function.name) {
            Some(context::Entry::Function(_, returns)) => returns.clone(),
            _ => panic!("[INTERNAL ERROR]: function should be bound in first pass"),
        };

        self.context.push();
        self.context.set_return(returns.clone());

        for parameter in &function.parameters {
            self.check_declaration(parameter)?;
        }

        if self.check_statement(&function.statements)? != r#type::Stm::Void && !returns.is_empty() {
            bail!(function.span, ErrorKind::MissingReturn);
        }

        self.context.pop();
        Ok(())
    }

    fn check_call(&self, call: &ast::Call) -> Result<Vec<r#type::Expression>, error::Error> {
        let (parameters, returns) = match self.context.get(call.name) {
            Some(context::Entry::Signature(parameters, returns))
            | Some(context::Entry::Function(parameters, returns)) => (parameters, returns),
            Some(_) => bail!(call.span, ErrorKind::NotFun(call.name)),
            None => bail!(call.span, ErrorKind::UnboundFun(call.name)),
        };

        if call.arguments.len() != parameters.len() {
            bail!(call.span, ErrorKind::CallLength);
        }

        for (argument, parameter) in call.arguments.iter().zip(parameters) {
            let r#type = self.check_expression(argument)?;

            if !r#type.subtypes(parameter) {
                expected!(argument.span(), parameter.clone(), r#type)
            }
        }

        Ok(returns.clone())
    }

    fn check_declaration(
        &mut self,
        declaration: &ast::Declaration,
    ) -> Result<r#type::Expression, error::Error> {
        if self.context.get(declaration.name).is_some() {
            bail!(declaration.span, ErrorKind::NameClash)
        }

        let r#type = self.check_type(&declaration.r#type)?;

        self.context
            .insert(declaration.name, context::Entry::Variable(r#type.clone()));

        Ok(r#type)
    }

    fn check_expression(&self, exp: &ast::Expression) -> Result<r#type::Expression, error::Error> {
        match exp {
            ast::Expression::Boolean(_, _) => Ok(r#type::Expression::Boolean),
            ast::Expression::Character(_, _) => Ok(r#type::Expression::Integer),
            ast::Expression::String(_, _) => Ok(r#type::Expression::Array(Box::new(
                r#type::Expression::Integer,
            ))),
            ast::Expression::Integer(_, _) => Ok(r#type::Expression::Integer),
            ast::Expression::Variable(name, span) => match self.context.get(*name) {
                Some(context::Entry::Variable(typ)) => Ok(typ.clone()),
                Some(_) => bail!(*span, ErrorKind::NotVariable(*name)),
                None => bail!(*span, ErrorKind::UnboundVariable(*name)),
            },

            ast::Expression::Array(array, _) => {
                let mut bound = r#type::Expression::Any;

                for expression in array {
                    let r#type = self.check_expression(expression)?;
                    match r#type.least_upper_bound(&bound) {
                        None => expected!(expression.span(), bound, r#type),
                        Some(_bound) => bound = _bound,
                    }
                }

                Ok(r#type::Expression::Array(Box::new(bound)))
            }

            ast::Expression::Binary(binary, left, right, _) => {
                match binary.get() {
                    ast::Binary::Add | ast::Binary::Cat => (),
                    ast::Binary::Mul
                    | ast::Binary::Hul
                    | ast::Binary::Div
                    | ast::Binary::Mod
                    | ast::Binary::Sub => {
                        return self.check_binary(
                            left,
                            right,
                            r#type::Expression::Integer,
                            r#type::Expression::Integer,
                        )
                    }
                    ast::Binary::Lt
                    | ast::Binary::Le
                    | ast::Binary::Ge
                    | ast::Binary::Gt
                    | ast::Binary::Ne
                    | ast::Binary::Eq => {
                        return self.check_binary(
                            left,
                            right,
                            r#type::Expression::Integer,
                            r#type::Expression::Boolean,
                        )
                    }
                    ast::Binary::And | ast::Binary::Or => {
                        return self.check_binary(
                            left,
                            right,
                            r#type::Expression::Boolean,
                            r#type::Expression::Boolean,
                        )
                    }
                }

                let span = right.span();

                if let (r#type::Expression::Array(left), r#type::Expression::Array(right)) =
                    (self.check_expression(left)?, self.check_expression(right)?)
                {
                    return match left.least_upper_bound(&right) {
                        None => expected!(span, *left, *right),
                        Some(bound) => {
                            binary.set(ast::Binary::Cat);
                            Ok(r#type::Expression::Array(Box::new(bound)))
                        }
                    };
                }

                self.check_binary(
                    left,
                    right,
                    r#type::Expression::Integer,
                    r#type::Expression::Integer,
                )
            }

            ast::Expression::Unary(ast::Unary::Neg, expression, _) => {
                match self.check_expression(expression)? {
                    r#type::Expression::Integer => Ok(r#type::Expression::Integer),
                    r#type => expected!(expression.span(), r#type::Expression::Integer, r#type),
                }
            }
            ast::Expression::Unary(ast::Unary::Not, expression, _) => {
                match self.check_expression(expression)? {
                    r#type::Expression::Boolean => Ok(r#type::Expression::Boolean),
                    r#type => expected!(expression.span(), r#type::Expression::Boolean, r#type),
                }
            }

            ast::Expression::Index(array, index, span) => {
                match (self.check_expression(array)?, self.check_expression(index)?) {
                    (r#type::Expression::Array(r#type), r#type::Expression::Integer)
                        if *r#type == r#type::Expression::Any =>
                    {
                        bail!(*span, ErrorKind::IndexEmpty)
                    }
                    (r#type::Expression::Array(r#type), r#type::Expression::Integer) => Ok(*r#type),
                    (r#type::Expression::Array(r#type), _) => {
                        expected!(index.span(), r#type::Expression::Integer, *r#type)
                    }
                    (r#type, _) => {
                        expected!(
                            array.span(),
                            r#type::Expression::Array(Box::new(r#type::Expression::Any)),
                            r#type
                        )
                    }
                }
            }

            ast::Expression::Call(call) if symbol::resolve(call.name) == "length" => {
                if call.arguments.len() != 1 {
                    bail!(call.span, ErrorKind::CallLength)
                }

                match self.check_expression(&call.arguments[0])? {
                    r#type::Expression::Array(_) => Ok(r#type::Expression::Integer),
                    typ => expected!(
                        call.span,
                        r#type::Expression::Array(Box::new(r#type::Expression::Any)),
                        typ
                    ),
                }
            }
            ast::Expression::Call(call) => {
                let mut returns = self.check_call(call)?;
                match returns.len() {
                    1 => Ok(returns.remove(0)),
                    _ => bail!(call.span, ErrorKind::NotExp),
                }
            }
        }
    }

    fn check_call_or_expression(
        &self,
        expression: &ast::Expression,
    ) -> Result<Vec<r#type::Expression>, error::Error> {
        match expression {
            ast::Expression::Call(call) if symbol::resolve(call.name) != "length" => {
                self.check_call(call)
            }
            expression => Ok(vec![self.check_expression(expression)?]),
        }
    }

    fn check_binary(
        &self,
        left: &ast::Expression,
        right: &ast::Expression,
        parameter: r#type::Expression,
        r#return: r#type::Expression,
    ) -> Result<r#type::Expression, error::Error> {
        match (self.check_expression(left)?, self.check_expression(right)?) {
            (left, right) if left.subtypes(&parameter) && right.subtypes(&parameter) => {
                Ok(r#return)
            }
            (left, mismatch) if left.subtypes(&parameter) => {
                expected!(right.span(), parameter, mismatch)
            }
            (mismatch, _) => expected!(left.span(), parameter, mismatch),
        }
    }

    fn check_statement(&mut self, statement: &ast::Statement) -> Result<r#type::Stm, error::Error> {
        match statement {
            ast::Statement::Assignment(left, right, _) => {
                let span = right.span();
                let left = self.check_expression(left)?;
                let right = self.check_expression(right)?;
                if right.subtypes(&left) {
                    Ok(r#type::Stm::Unit)
                } else {
                    expected!(span, left, right)
                }
            }
            ast::Statement::Call(call) => match &*self.check_call(call)? {
                [] => Ok(r#type::Stm::Unit),
                _ => bail!(call.span, ErrorKind::NotProcedure),
            },
            ast::Statement::Declaration(declaration, _) => {
                self.check_declaration(declaration)?;
                Ok(r#type::Stm::Unit)
            }
            ast::Statement::Initialization(declarations, expression, span) => {
                let initializations = self.check_call_or_expression(expression)?;

                if initializations.is_empty() {
                    bail!(*span, ErrorKind::InitProcedure);
                }

                if initializations.len() != declarations.len() {
                    bail!(*span, ErrorKind::InitLength);
                }

                for (declaration, initialization) in declarations.iter().zip(initializations) {
                    if let Some(declaration) = declaration {
                        let r#type = self.check_declaration(declaration)?;
                        if !initialization.subtypes(&r#type) {
                            expected!(declaration.span, initialization, r#type);
                        }
                    }
                }

                Ok(r#type::Stm::Unit)
            }
            ast::Statement::Return(returns, span) => {
                let returns = returns
                    .iter()
                    .map(|r#return| self.check_expression(r#return))
                    .collect::<Result<Vec<_>, _>>()?;

                let expected = self.context.get_returns();

                if r#type::subtypes(&returns, expected) {
                    Ok(r#type::Stm::Void)
                } else {
                    bail!(*span, ErrorKind::ReturnMismatch);
                }
            }
            ast::Statement::Sequence(statements, _) => {
                self.context.push();

                let mut r#type = r#type::Stm::Unit;

                for statement in statements {
                    if r#type == r#type::Stm::Void {
                        bail!(statement.span(), ErrorKind::Unreachable)
                    } else if self.check_statement(statement)? == r#type::Stm::Void {
                        r#type = r#type::Stm::Void;
                    }
                }

                self.context.pop();
                Ok(r#type)
            }
            ast::Statement::If(condition, r#if, None, _) => {
                match self.check_expression(condition)? {
                    r#type::Expression::Boolean => (),
                    typ => expected!(condition.span(), r#type::Expression::Boolean, typ),
                };

                self.context.push();
                self.check_statement(r#if)?;
                self.context.pop();

                Ok(r#type::Stm::Unit)
            }
            ast::Statement::If(condition, r#if, Some(r#else), _) => {
                match self.check_expression(condition)? {
                    r#type::Expression::Boolean => (),
                    typ => expected!(condition.span(), r#type::Expression::Boolean, typ),
                };

                self.context.push();
                let r#if = self.check_statement(r#if)?;
                self.context.pop();

                self.context.push();
                let r#else = self.check_statement(r#else)?;
                self.context.pop();

                Ok(r#if.least_upper_bound(&r#else))
            }
            ast::Statement::While(cond, body, _) => {
                match self.check_expression(cond)? {
                    r#type::Expression::Boolean => (),
                    typ => expected!(cond.span(), r#type::Expression::Boolean, typ),
                };

                self.context.push();
                self.check_statement(body)?;
                self.context.pop();

                Ok(r#type::Stm::Unit)
            }
        }
    }
}
