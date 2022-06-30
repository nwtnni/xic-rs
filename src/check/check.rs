use std::path::Path;

use crate::abi;
use crate::check::context::Entry;
use crate::check::context::GlobalScope;
use crate::check::context::LeastUpperBound;
use crate::check::context::LocalScope;
use crate::check::Context;
use crate::check::Error;
use crate::check::ErrorKind;
use crate::check::Scope;
use crate::data::ast;
use crate::data::r#type;
use crate::data::symbol::Symbol;
use crate::util;
use crate::Set;

macro_rules! bail {
    ($span:expr, $kind:expr $(,)?) => {
        return Err(Error::new($span, $kind).into())
    };
}

macro_rules! expected {
    ($expected_span:expr, $expected:expr, $found_span:expr, $found:expr $(,)?) => {{
        let kind = ErrorKind::Mismatch {
            expected: $expected,
            expected_span: Some($expected_span),
            found: $found,
        };
        bail!($found_span, kind)
    }};

    ($expected:expr, $found_span:expr, $found:expr $(,)?) => {{
        let kind = ErrorKind::Mismatch {
            expected: $expected,
            expected_span: None,
            found: $found,
        };
        bail!($found_span, kind)
    }};
}

pub fn check(
    directory_library: Option<&Path>,
    path: &Path,
    mut program: ast::Program<()>,
) -> Result<(ast::Program<r#type::Expression>, Context), crate::Error> {
    log::info!(
        "[{}] Type checking {}...",
        std::any::type_name::<ast::Program<()>>(),
        path.display(),
    );
    util::time!(
        "[{}] Done type checking {}",
        std::any::type_name::<ast::Program<()>>(),
        path.display(),
    );

    let mut checker = Checker {
        context: Context::new(),
        used: Set::default(),
    };

    let directory_library = directory_library.unwrap_or_else(|| path.parent().unwrap());

    checker.load_program(directory_library, path, &program)?;
    checker.monomorphize_program(&mut program)?;
    let program = checker.check_program(program)?;

    Ok((program, checker.context))
}

pub(super) struct Checker {
    pub(super) context: Context,

    /// Set of unique interfaces in the use tree
    pub(super) used: Set<Symbol>,
}

impl Checker {
    fn check_program(
        &mut self,
        program: ast::Program<()>,
    ) -> Result<ast::Program<r#type::Expression>, Error> {
        let items = program
            .items
            .into_iter()
            .map(|item| match item {
                ast::Item::Global(global) => self.check_global(global).map(ast::Item::Global),
                ast::Item::Class(class) => {
                    let provenance = class.provenance.clone();
                    self.check_class(class)
                        .map_err(|error| error.with_provenance(provenance))
                        .map(ast::Item::Class)
                }
                ast::Item::ClassTemplate(_) => unreachable!(),
                ast::Item::Function(function) => {
                    let provenance = function.provenance.clone();
                    self.check_function(GlobalScope::Global, function)
                        .map_err(|error| error.with_provenance(provenance))
                        .map(ast::Item::Function)
                }
                ast::Item::FunctionTemplate(_) => unreachable!(),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ast::Program {
            uses: program.uses,
            items,
        })
    }

    fn check_global(
        &mut self,
        global: ast::Global<()>,
    ) -> Result<ast::Global<r#type::Expression>, Error> {
        match global {
            ast::Global::Initialization(initialization) => self
                .check_initialization(GlobalScope::Global, initialization)
                .map(ast::Global::Initialization),
            ast::Global::Declaration(declaration) => self
                .check_declaration(GlobalScope::Global, declaration)
                .map(ast::Global::Declaration),
        }
    }

    pub(super) fn check_class_signature(
        &mut self,
        class: &ast::ClassSignature<()>,
    ) -> Result<(), Error> {
        let _ = self.check_class_like(class)?;

        for method in &class.methods {
            let _ = self.check_function_like(method)?;
        }

        Ok(())
    }

    fn check_class(
        &mut self,
        class: ast::Class<()>,
    ) -> Result<ast::Class<r#type::Expression>, Error> {
        let extends = self.check_class_like(&class)?;

        // Classes must implement at least the methods declared in its interface
        if let Some(span) = self
            .context
            .get_class(&class.name)
            .unwrap()
            .iter()
            .find_map(|(identifier, entry)| match entry {
                Entry::Signature(_, _) => Some(*identifier.span),
                _ => None,
            })
        {
            bail!(
                class.span,
                ErrorKind::ClassIncomplete(class.name.symbol, span)
            );
        }

        let items = class
            .items
            .into_iter()
            .map(|item| {
                match item {
                    ast::ClassItem::Field(declaration) => self
                        .check_declaration(GlobalScope::Class(class.name.symbol), declaration)
                        .map(ast::ClassItem::Field),
                    ast::ClassItem::Method(method) => {
                        // Check if method is declared or defined by an ancestor
                        if let Some((
                            span,
                            Entry::Signature(old_parameters, old_returns)
                            | Entry::Function(old_parameters, old_returns),
                        )) = self
                            .context
                            .ancestors_exclusive(&class.name.symbol)
                            .find_map(|class| {
                                self.context.get_class(&class).unwrap().get(&method.name)
                            })
                        {
                            let (new_parameters, new_returns) = self
                                .context
                                .get_class(&class.name)
                                .unwrap()
                                .get(&method.name)
                                .and_then(|(_, entry)| match entry {
                                    Entry::Function(new_parameters, new_returns) => {
                                        Some((new_parameters, new_returns))
                                    }
                                    _ => None,
                                })
                                .unwrap();

                            if !self.context.all_subtype(old_parameters, new_parameters)
                                || !self.context.all_subtype(new_returns, old_returns)
                            {
                                bail!(*method.name.span, ErrorKind::SignatureMismatch(*span));
                            }
                        }

                        self.check_function(GlobalScope::Class(class.name.symbol), method)
                            .map(ast::ClassItem::Method)
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ast::Class {
            r#final: class.r#final,
            name: class.name,
            extends,
            items,
            provenance: class.provenance,
            declared: class.declared.clone(),
            span: class.span,
        })
    }

    fn check_class_like<C: ast::ClassLike<()>>(
        &mut self,
        class: &C,
    ) -> Result<Option<ast::Variable<r#type::Expression>>, Error> {
        if let Some(supertype) = class.extends() {
            let supertype = self.check_variable(supertype.clone())?;
            if let Some(span) = self.context.get_final(&supertype.name.symbol) {
                bail!(
                    supertype.span,
                    ErrorKind::FinalSuperclass(supertype.name.symbol, *span)
                );
            }

            Ok(Some(supertype))
        } else {
            Ok(None)
        }
    }

    fn check_function(
        &mut self,
        scope: GlobalScope,
        function: ast::Function<()>,
    ) -> Result<ast::Function<r#type::Expression>, Error> {
        let returns = self.check_function_like(&function)?;

        let is_procedure = returns.is_empty();
        let scope = match scope {
            GlobalScope::Class(class) => LocalScope::Method {
                class,
                this: None,
                returns: returns.iter().map(|r#return| r#return.r#type()).collect(),
            },
            GlobalScope::Global => LocalScope::Function {
                returns: returns.iter().map(|r#return| r#return.r#type()).collect(),
            },
        };

        self.context.push(scope);

        let parameters = function
            .parameters
            .into_iter()
            .map(|parameter| self.check_single_declaration(Scope::Local, parameter))
            .collect::<Result<Vec<_>, _>>()?;

        let (statements, statements_type) = self.check_statement(function.statements)?;

        if statements_type != r#type::Statement::Void && !is_procedure {
            bail!(function.span, ErrorKind::MissingReturn);
        }

        self.context.pop();

        Ok(ast::Function {
            name: function.name,
            parameters,
            returns,
            statements,
            declared: function.declared,
            provenance: function.provenance,
            span: function.span,
        })
    }

    pub(super) fn check_function_like<C: ast::FunctionLike<()>>(
        &mut self,
        function: &C,
    ) -> Result<Vec<ast::Type<r#type::Expression>>, Error> {
        function
            .parameters()
            .iter()
            .map(|parameter| &*parameter.r#type)
            .cloned()
            .try_for_each(|parameter| {
                self.check_type(parameter)?;
                Ok(())
            })?;

        function
            .returns()
            .iter()
            .cloned()
            .map(|r#return| self.check_type(r#return))
            .collect()
    }

    fn check_statement(
        &mut self,
        statement: ast::Statement<()>,
    ) -> Result<(ast::Statement<r#type::Expression>, r#type::Statement), Error> {
        match statement {
            ast::Statement::Assignment(left, right, span) => {
                let left_span = left.span();
                let right_span = right.span();

                let left = self.check_expression(*left).map(Box::new)?;
                let left_type = left.r#type();

                let right = self.check_expression(*right).map(Box::new)?;
                let right_type = right.r#type();

                if self.context.is_subtype(&right_type, &left_type) {
                    Ok((
                        ast::Statement::Assignment(left, right, span),
                        r#type::Statement::Unit,
                    ))
                } else {
                    expected!(left_span, left_type, right_span, right_type)
                }
            }
            ast::Statement::Call(call) => {
                let call = self.check_call(call)?;

                match call.function.r#type() {
                    r#type::Expression::Function(_, returns) if returns.is_empty() => {
                        Ok((ast::Statement::Call(call), r#type::Statement::Unit))
                    }
                    r#type::Expression::Function(_, _) => bail!(call.span, ErrorKind::NotProcedure),
                    _ => unreachable!(),
                }
            }
            ast::Statement::Declaration(declaration, span) => self
                .check_declaration(Scope::Local, *declaration)
                .map(Box::new)
                .map(|declaration| ast::Statement::Declaration(declaration, span))
                .map(|declaration| (declaration, r#type::Statement::Unit)),
            ast::Statement::Initialization(initialization) => {
                let initialization = self.check_initialization(Scope::Local, initialization)?;
                Ok((
                    ast::Statement::Initialization(initialization),
                    r#type::Statement::Unit,
                ))
            }
            ast::Statement::Return(returns, span) => {
                let returns = returns
                    .into_iter()
                    .map(|r#return| self.check_expression(r#return))
                    .collect::<Result<Vec<_>, _>>()?;

                let expected = self
                    .context
                    .get_scoped_returns()
                    .expect("[INTERNAL PARSER ERROR]: return outside function scope");

                let actual = returns
                    .iter()
                    .map(|r#return| r#return.r#type())
                    .collect::<Vec<_>>();

                if self.context.all_subtype(&actual, expected) {
                    Ok((
                        ast::Statement::Return(returns, span),
                        r#type::Statement::Void,
                    ))
                } else {
                    bail!(span, ErrorKind::ReturnMismatch);
                }
            }
            ast::Statement::Sequence(statements, span) => {
                self.context.push(LocalScope::Block);

                let mut r#type = r#type::Statement::Unit;

                let statements = statements
                    .into_iter()
                    .map(|statement| {
                        let (statement, statement_type) = self.check_statement(statement)?;
                        if r#type == r#type::Statement::Void {
                            bail!(statement.span(), ErrorKind::Unreachable)
                        } else if statement_type == r#type::Statement::Void {
                            r#type = r#type::Statement::Void;
                        }
                        Ok(statement)
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                self.context.pop();
                Ok((ast::Statement::Sequence(statements, span), r#type))
            }
            ast::Statement::If(condition, r#if, None, span) => {
                let condition = self.check_expression(*condition).map(Box::new)?;
                match condition.r#type() {
                    r#type::Expression::Boolean => (),
                    r#type => expected!(r#type::Expression::Boolean, condition.span(), r#type),
                }

                self.context.push(LocalScope::If);
                let (r#if, _) = self.check_statement(*r#if)?;
                self.context.pop();

                Ok((
                    ast::Statement::If(condition, Box::new(r#if), None, span),
                    r#type::Statement::Unit,
                ))
            }
            ast::Statement::If(condition, r#if, Some(r#else), span) => {
                let condition = self.check_expression(*condition).map(Box::new)?;
                match condition.r#type() {
                    r#type::Expression::Boolean => (),
                    r#type => expected!(r#type::Expression::Boolean, condition.span(), r#type),
                }

                self.context.push(LocalScope::If);
                let (r#if, if_type) = self.check_statement(*r#if)?;
                self.context.pop();

                self.context.push(LocalScope::Else);
                let (r#else, else_type) = self.check_statement(*r#else)?;
                self.context.pop();

                Ok((
                    ast::Statement::If(condition, Box::new(r#if), Some(Box::new(r#else)), span),
                    if_type.least_upper_bound(&else_type),
                ))
            }
            ast::Statement::While(r#do, condition, body, span) => {
                let condition = self.check_expression(*condition).map(Box::new)?;
                match condition.r#type() {
                    r#type::Expression::Boolean => (),
                    r#type => expected!(r#type::Expression::Boolean, condition.span(), r#type),
                }

                self.context.push(LocalScope::While(None));
                let (body, _) = self.check_statement(*body)?;
                self.context.pop();

                Ok((
                    ast::Statement::While(r#do, condition, Box::new(body), span),
                    r#type::Statement::Unit,
                ))
            }
            ast::Statement::Break(span) => match self.context.get_scoped_while() {
                None => bail!(span, ErrorKind::NotInWhile),
                Some(_) => Ok((ast::Statement::Break(span), r#type::Statement::Void)),
            },
        }
    }

    fn check_expression(
        &self,
        expression: ast::Expression<()>,
    ) -> Result<ast::Expression<r#type::Expression>, Error> {
        match expression {
            ast::Expression::Boolean(boolean, span) => Ok(ast::Expression::Boolean(boolean, span)),
            ast::Expression::Character(character, span) => {
                Ok(ast::Expression::Character(character, span))
            }
            ast::Expression::String(string, span) => Ok(ast::Expression::String(string, span)),
            ast::Expression::Integer(integer, span) => Ok(ast::Expression::Integer(integer, span)),
            ast::Expression::Null(span) => Ok(ast::Expression::Null(span)),
            ast::Expression::This((), span) => self
                .context
                .get_scoped_class()
                .map(r#type::Expression::Class)
                .map(|r#type| ast::Expression::This(r#type, span))
                .ok_or_else(|| Error::new(span, ErrorKind::NotInClass(None))),
            ast::Expression::Super(_, span) => self
                .context
                .get_scoped_class()
                .ok_or(ErrorKind::NotInClass(None))
                .and_then(|class| {
                    self.context
                        .get_superclass(&class)
                        .ok_or(ErrorKind::NoSuperclass(class))
                })
                .map(r#type::Expression::Class)
                .map(|r#type| ast::Expression::Super(r#type, span))
                .map_err(|kind| Error::new(span, kind)),
            ast::Expression::Variable(
                ast::Variable {
                    name,
                    generics,
                    span,
                },
                (),
            ) => {
                assert!(generics.is_none());
                match self.context.get(Scope::Local, &name) {
                    Some(Entry::Variable(r#type)) => Ok(ast::Expression::Variable(
                        ast::Variable {
                            name,
                            generics: None,
                            span,
                        },
                        r#type.clone(),
                    )),
                    Some(_) => bail!(*name.span, ErrorKind::NotVariable(name.symbol)),
                    None => bail!(*name.span, ErrorKind::UnboundVariable(name.symbol)),
                }
            }

            ast::Expression::Array(expressions, (), span) => {
                let mut bound = r#type::Expression::Any;
                let mut bound_span = None;

                expressions
                    .into_iter()
                    .map(|expression| {
                        let expression = self.check_expression(expression)?;
                        let r#type = expression.r#type();

                        match self.context.least_upper_bound(&bound, &r#type) {
                            None => {
                                expected!(
                                    bound_span.unwrap(),
                                    bound.clone(),
                                    expression.span(),
                                    r#type
                                )
                            }
                            Some(LeastUpperBound::Left(_bound)) => bound = _bound,
                            Some(LeastUpperBound::Right(_bound)) => {
                                bound = _bound;
                                bound_span = Some(expression.span());
                            }
                        }

                        Ok(expression)
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(|expressions| {
                        ast::Expression::Array(
                            expressions,
                            r#type::Expression::Array(Box::new(bound)),
                            span,
                        )
                    })
            }

            ast::Expression::Binary(binary, left, right, (), span) => {
                let left_span = left.span();
                let right_span = right.span();

                let left = self.check_expression(*left).map(Box::new)?;
                let left_type = left.r#type();

                let right = self.check_expression(*right).map(Box::new)?;
                let right_type = right.r#type();

                let (parameter, r#return) = match binary {
                    // Note: array concatenation handled specially below
                    ast::Binary::Cat
                    | ast::Binary::Add
                    | ast::Binary::Mul
                    | ast::Binary::Hul
                    | ast::Binary::Div
                    | ast::Binary::Mod
                    | ast::Binary::Sub => {
                        (r#type::Expression::Integer, r#type::Expression::Integer)
                    }
                    ast::Binary::Lt | ast::Binary::Le | ast::Binary::Ge | ast::Binary::Gt => {
                        (r#type::Expression::Integer, r#type::Expression::Boolean)
                    }
                    ast::Binary::And | ast::Binary::Or => {
                        (r#type::Expression::Boolean, r#type::Expression::Boolean)
                    }
                    ast::Binary::Ne | ast::Binary::Eq => {
                        if self.context.is_subtype(&left_type, &right_type)
                            || self.context.is_subtype(&right_type, &left_type)
                        {
                            return Ok(ast::Expression::Binary(
                                binary,
                                left,
                                right,
                                r#type::Expression::Boolean,
                                span,
                            ));
                        } else {
                            expected!(left_span, left_type, right_span, right_type);
                        }
                    }
                };

                if let (
                    ast::Binary::Add | ast::Binary::Cat,
                    r#type::Expression::Array(_),
                    r#type::Expression::Array(_),
                ) = (binary, &left_type, &right_type)
                {
                    return match self.context.least_upper_bound(&left_type, &right_type) {
                        None => expected!(left_span, left_type, right_span, right_type),
                        Some(LeastUpperBound::Left(r#type) | LeastUpperBound::Right(r#type)) => Ok(
                            ast::Expression::Binary(ast::Binary::Cat, left, right, r#type, span),
                        ),
                    };
                }

                if self.context.is_subtype(&left_type, &parameter)
                    && self.context.is_subtype(&right_type, &parameter)
                {
                    Ok(ast::Expression::Binary(binary, left, right, r#return, span))
                } else if self.context.is_subtype(&left_type, &parameter) {
                    expected!(parameter, right_span, right_type)
                } else {
                    expected!(parameter, left_span, left_type)
                }
            }

            ast::Expression::Unary(ast::Unary::Neg, expression, (), span) => {
                let expression = self.check_expression(*expression).map(Box::new)?;
                match expression.r#type() {
                    r#type::Expression::Integer => Ok(ast::Expression::Unary(
                        ast::Unary::Neg,
                        expression,
                        r#type::Expression::Integer,
                        span,
                    )),
                    r#type => expected!(r#type::Expression::Integer, expression.span(), r#type),
                }
            }
            ast::Expression::Unary(ast::Unary::Not, expression, (), span) => {
                let expression = self.check_expression(*expression).map(Box::new)?;
                match expression.r#type() {
                    r#type::Expression::Boolean => Ok(ast::Expression::Unary(
                        ast::Unary::Not,
                        expression,
                        r#type::Expression::Boolean,
                        span,
                    )),
                    r#type => expected!(r#type::Expression::Boolean, expression.span(), r#type),
                }
            }

            ast::Expression::Index(array, index, (), span) => {
                let array = self.check_expression(*array).map(Box::new)?;
                let index = self.check_expression(*index).map(Box::new)?;
                match (array.r#type(), index.r#type()) {
                    (r#type::Expression::Array(r#type), r#type::Expression::Integer)
                        if *r#type == r#type::Expression::Any =>
                    {
                        bail!(span, ErrorKind::IndexEmpty)
                    }
                    (r#type::Expression::Array(r#type), r#type::Expression::Integer) => {
                        Ok(ast::Expression::Index(array, index, *r#type, span))
                    }
                    (r#type::Expression::Array(r#type), _) => {
                        expected!(r#type::Expression::Integer, index.span(), *r#type)
                    }
                    (r#type, _) => {
                        expected!(
                            r#type::Expression::Array(Box::new(r#type::Expression::Any)),
                            array.span(),
                            r#type,
                        )
                    }
                }
            }
            ast::Expression::Length(array, span) => {
                let array = self.check_expression(*array).map(Box::new)?;
                match array.r#type() {
                    r#type::Expression::Array(_) => Ok(ast::Expression::Length(array, span)),
                    r#type => expected!(
                        r#type::Expression::Array(Box::new(r#type::Expression::Any)),
                        span,
                        r#type,
                    ),
                }
            }

            ast::Expression::Dot(receiver, field, (), span) => {
                let receiver = self.check_expression(*receiver).map(Box::new)?;
                let class = match receiver.r#type() {
                    r#type::Expression::Class(class) => class,
                    _ => bail!(receiver.span(), ErrorKind::NotClass),
                };

                match self.context.get(GlobalScope::Class(class), &field) {
                    None => bail!(*field.span, ErrorKind::UnboundVariable(field.symbol)),
                    Some(Entry::Variable(r#type)) => {
                        Ok(ast::Expression::Dot(receiver, field, r#type.clone(), span))
                    }
                    Some(_) => bail!(*field.span, ErrorKind::NotVariable(field.symbol)),
                }
            }
            ast::Expression::New(
                ast::Variable {
                    name,
                    generics,
                    span: span_,
                },
                span,
            ) => {
                assert!(generics.is_none());
                match self.context.get_class_implementation(&name) {
                    Some(_) => Ok(ast::Expression::New(
                        ast::Variable {
                            name,
                            generics: None,
                            span: span_,
                        },
                        span,
                    )),
                    None if self.context.get_class(&name).is_some() => {
                        bail!(span, ErrorKind::NotInClassModule(name.symbol))
                    }
                    None => {
                        bail!(*name.span, ErrorKind::UnboundClass(name.symbol))
                    }
                }
            }
            ast::Expression::Call(call) => {
                let call = self.check_call(call)?;
                match call.function.r#type() {
                    r#type::Expression::Function(_, returns) if returns.len() == 1 => {
                        Ok(ast::Expression::Call(call))
                    }
                    _ => bail!(call.span, ErrorKind::NotExp),
                }
            }
        }
    }

    fn check_call(&self, call: ast::Call<()>) -> Result<ast::Call<r#type::Expression>, Error> {
        let (scope, function, function_name, function_span): (
            _,
            Box<dyn FnOnce(r#type::Expression) -> ast::Expression<r#type::Expression>>,
            _,
            _,
        ) = match *call.function {
            ast::Expression::Variable(
                ast::Variable {
                    name,
                    generics,
                    span,
                },
                (),
            ) => {
                assert!(generics.is_none());

                let function_name = name.symbol;
                let function_span = *name.span;

                (
                    Scope::Local,
                    Box::new(move |r#type| {
                        ast::Expression::Variable(
                            ast::Variable {
                                name,
                                generics: None,
                                span,
                            },
                            r#type,
                        )
                    }) as _,
                    function_name,
                    function_span,
                )
            }
            ast::Expression::Dot(receiver, name, (), span) => {
                let receiver = self.check_expression(*receiver).map(Box::new)?;
                let class = match receiver.r#type() {
                    r#type::Expression::Class(class) => class,
                    _ => bail!(receiver.span(), ErrorKind::NotClass),
                };

                let function_name = name.symbol;
                let function_span = span;

                (
                    Scope::Global(GlobalScope::Class(class)),
                    Box::new(move |r#type| ast::Expression::Dot(receiver, name, r#type, span)) as _,
                    function_name,
                    function_span,
                )
            }
            expression => bail!(expression.span(), ErrorKind::NotFun(None)),
        };

        let (parameters, returns) = match self.context.get(scope, &function_name) {
            Some(Entry::Signature(parameters, returns))
            | Some(Entry::Function(parameters, returns)) => (parameters, returns),
            Some(_) => bail!(function_span, ErrorKind::NotFun(Some(function_name))),
            None => bail!(function_span, ErrorKind::UnboundFun(function_name)),
        };

        if call.arguments.len() != parameters.len() {
            bail!(call.span, ErrorKind::CallLength);
        }

        let arguments = call
            .arguments
            .into_iter()
            .zip(parameters)
            .map(|(argument, parameter)| {
                let argument = self.check_expression(argument)?;
                let r#type = argument.r#type();

                if self.context.is_subtype(&r#type, parameter) {
                    Ok(argument)
                } else {
                    // TODO: attach span to parameters
                    expected!(parameter.clone(), argument.span(), r#type)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ast::Call {
            function: Box::new(function(r#type::Expression::Function(
                parameters.clone(),
                returns.clone(),
            ))),
            arguments,
            span: call.span,
        })
    }

    fn check_initialization<S: Into<Scope>>(
        &mut self,
        scope: S,
        ast::Initialization {
            declarations,
            expression,
            span,
        }: ast::Initialization<()>,
    ) -> Result<ast::Initialization<r#type::Expression>, Error> {
        let (expression, r#types) = match *expression {
            ast::Expression::Call(call) => {
                let call = self.check_call(call)?;
                let returns = match call.function.r#type() {
                    r#type::Expression::Function(_, returns) => returns,
                    _ => unreachable!(),
                };
                (ast::Expression::Call(call), returns)
            }
            expression => {
                let expression = self.check_expression(expression)?;
                let r#type = expression.r#type();
                (expression, vec![r#type])
            }
        };

        if r#types.is_empty() {
            bail!(span, ErrorKind::InitProcedure);
        }

        if r#types.len() != declarations.len() {
            bail!(span, ErrorKind::InitLength);
        }

        let scope = scope.into();

        let declarations = declarations
            .into_iter()
            .zip(r#types)
            .map(|(declaration, subtype)| {
                let declaration = match declaration {
                    Some(declaration) => declaration,
                    None => return Ok(None),
                };

                let declaration = match self.check_single_declaration(scope, declaration) {
                    Ok(declaration) => declaration,
                    Err(error) => return Err(error),
                };

                let supertype = declaration.r#type.r#type();
                if !self.context.is_subtype(&subtype, &supertype) {
                    expected!(declaration.span(), supertype, expression.span(), subtype);
                }

                Ok(Some(declaration))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ast::Initialization {
            declarations,
            expression: Box::new(expression),
            span,
        })
    }

    fn check_declaration<S: Into<Scope>>(
        &mut self,
        scope: S,
        declaration: ast::Declaration<()>,
    ) -> Result<ast::Declaration<r#type::Expression>, Error> {
        let multiple = match declaration {
            ast::Declaration::Multiple(multiple) => multiple,
            ast::Declaration::Single(single) => {
                return self
                    .check_single_declaration(scope, single)
                    .map(ast::Declaration::Single);
            }
        };

        assert!(!multiple.r#type.has_length());

        let scope = scope.into();
        for name in &multiple.names {
            self.check_single_declaration(
                scope,
                ast::SingleDeclaration {
                    name: name.clone(),
                    r#type: multiple.r#type.clone(),
                    span: multiple.span,
                },
            )?;
        }

        Ok(ast::Declaration::Multiple(ast::MultipleDeclaration {
            names: multiple.names,
            r#type: self.check_type(*multiple.r#type).map(Box::new)?,
            span: multiple.span,
        }))
    }

    fn check_single_declaration<S: Into<Scope>>(
        &mut self,
        scope: S,
        ast::SingleDeclaration { name, r#type, span }: ast::SingleDeclaration<()>,
    ) -> Result<ast::SingleDeclaration<r#type::Expression>, Error> {
        let r#type = self.check_type(*r#type).map(Box::new)?;
        let scope = scope.into();

        match (
            scope,
            self.context
                .insert(scope, name.clone(), Entry::Variable(r#type.r#type())),
        ) {
            (_, None) => (),
            // Note: class fields are inserted during loading, since they need
            // to be visible everywhere.
            (Scope::Global(GlobalScope::Class(_)), Some(_)) => (),
            (Scope::Local | Scope::Global(GlobalScope::Global), Some((span, _))) => {
                bail!(*name.span, ErrorKind::NameClash(span))
            }
        }

        Ok(ast::SingleDeclaration { name, r#type, span })
    }

    pub(super) fn check_type(
        &self,
        r#type: ast::Type<()>,
    ) -> Result<ast::Type<r#type::Expression>, Error> {
        match r#type {
            ast::Type::Bool(span) => Ok(ast::Type::Bool(span)),
            ast::Type::Int(span) => Ok(ast::Type::Int(span)),
            ast::Type::Array(r#type, None, span) => self
                .check_type(*r#type)
                .map(Box::new)
                .map(|r#type| ast::Type::Array(r#type, None, span)),
            ast::Type::Class(variable) => self.check_variable(variable).map(ast::Type::Class),
            ast::Type::Array(r#type, Some(length), span) => {
                let r#type = self.check_type(*r#type).map(Box::new)?;
                let length = self.check_expression(*length).map(Box::new)?;
                match length.r#type() {
                    r#type::Expression::Integer => Ok(ast::Type::Array(r#type, Some(length), span)),
                    r#type => expected!(r#type::Expression::Integer, length.span(), r#type),
                }
            }
        }
    }

    // There are three cases where this function is called:
    //
    // 1) Checking interfaces during the second half of the loading pass.
    //
    // Here, we haven't monomorphized yet, so this branch is reachable,
    // and type arguments should be checked. But we *don't* want to check
    // that the template instantiation exists, because its implementation
    // may be in a separate compilation unit.
    //
    // 2) Checking template instantiation sites during monomorphization
    //
    // Here, we want to catch unbound templates and type parameters.
    // But for compatibility with (1), we won't check that the template
    // instantiation exists--we'll assume the monomorphization pass
    // correctly generates it.
    //
    // 3) Checking signatures after monomorphization.
    //
    // Here, there are no more generics, so this branch is unreachable.
    // Any unbound classes within the type arguments must be caught
    // during the monomorphization pass.
    fn check_variable(
        &self,
        variable: ast::Variable<()>,
    ) -> Result<ast::Variable<r#type::Expression>, Error> {
        match variable.generics {
            None => match self.context.get_class(&variable.name) {
                Some(_) => Ok(ast::Variable {
                    name: variable.name,
                    generics: None,
                    span: variable.span,
                }),
                None => bail!(
                    *variable.name.span,
                    ErrorKind::UnboundClass(variable.name.symbol)
                ),
            },
            Some(generics) => {
                match self.context.get_class_template(&variable.name) {
                    None => {
                        bail!(
                            *variable.name.span,
                            ErrorKind::UnboundClassTemplate(variable.name.symbol)
                        )
                    }
                    Some(template) if template.generics.len() != generics.len() => bail!(
                        variable.span,
                        ErrorKind::TemplateArgumentMismatch {
                            span: *template.name.span,
                            expected: template.generics.len(),
                            found: generics.len()
                        },
                    ),
                    Some(_) => (),
                }

                let generics = generics
                    .into_iter()
                    .map(|generic| self.check_type(generic))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(ast::Variable {
                    name: ast::Identifier {
                        symbol: abi::mangle::template(&variable.name.symbol, &generics),
                        span: variable.name.span,
                    },
                    generics: None,
                    span: variable.span,
                })
            }
        }
    }
}

impl ast::Expression<r#type::Expression> {
    pub(crate) fn r#type(&self) -> r#type::Expression {
        match self {
            ast::Expression::Boolean(_, _) => r#type::Expression::Boolean,
            ast::Expression::Character(_, _)
            | ast::Expression::Integer(_, _)
            | ast::Expression::Length(_, _) => r#type::Expression::Integer,
            ast::Expression::String(_, _) => {
                r#type::Expression::Array(Box::new(r#type::Expression::Integer))
            }
            ast::Expression::Null(_) => r#type::Expression::Null,
            ast::Expression::This(r#type, _)
            | ast::Expression::Super(r#type, _)
            | ast::Expression::Variable(_, r#type)
            | ast::Expression::Array(_, r#type, _)
            | ast::Expression::Binary(_, _, _, r#type, _)
            | ast::Expression::Unary(_, _, r#type, _)
            | ast::Expression::Index(_, _, r#type, _)
            | ast::Expression::Dot(_, _, r#type, _) => r#type.clone(),
            ast::Expression::New(class, _) => {
                assert!(class.generics.is_none());
                r#type::Expression::Class(class.name.symbol)
            }
            ast::Expression::Call(call) => match call.function.r#type() {
                r#type::Expression::Function(_, returns) if returns.len() == 1 => {
                    returns.first().unwrap().clone()
                }
                _ => unreachable!(),
            },
        }
    }
}

impl ast::Type<r#type::Expression> {
    pub(crate) fn r#type(&self) -> r#type::Expression {
        match self {
            ast::Type::Bool(_) => r#type::Expression::Boolean,
            ast::Type::Int(_) => r#type::Expression::Integer,
            ast::Type::Class(variable) => {
                assert!(variable.generics.is_none());
                r#type::Expression::Class(variable.name.symbol)
            }
            ast::Type::Array(r#type, _, _) => r#type::Expression::Array(Box::new(r#type.r#type())),
        }
    }
}
