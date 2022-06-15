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
    program: &mut ast::Program,
) -> Result<Context, crate::Error> {
    log::info!(
        "[{}] Type checking {}...",
        std::any::type_name::<ast::Program>(),
        path.display(),
    );
    util::time!(
        "[{}] Done type checking {}",
        std::any::type_name::<ast::Program>(),
        path.display(),
    );

    let mut checker = Checker {
        context: Context::new(),
        used: Set::default(),
    };

    let directory_library = directory_library.unwrap_or_else(|| path.parent().unwrap());

    checker.load_program(directory_library, path, program)?;
    checker.monomorphize_program(program);
    checker.check_program(program)?;

    Ok(checker.context)
}

pub(super) struct Checker {
    pub(super) context: Context,

    /// Set of unique interfaces in the use tree
    pub(super) used: Set<Symbol>,
}

impl Checker {
    fn check_program(&mut self, program: &ast::Program) -> Result<(), Error> {
        for item in &program.items {
            match item {
                ast::Item::Global(global) => self.check_global(global)?,
                ast::Item::Class(class) => self
                    .check_class(class)
                    .map_err(|error| error.with_provenance(class.provenance.clone()))?,
                ast::Item::ClassTemplate(_) => unreachable!(),
                ast::Item::Function(function) => self
                    .check_function(GlobalScope::Global, function)
                    .map_err(|error| error.with_provenance(function.provenance.clone()))?,
                ast::Item::FunctionTemplate(_) => unreachable!(),
            }
        }

        Ok(())
    }

    fn check_global(&mut self, global: &ast::Global) -> Result<(), Error> {
        match global {
            ast::Global::Initialization(initialization) => {
                self.check_initialization(GlobalScope::Global, initialization)
            }
            ast::Global::Declaration(declaration) => {
                self.check_declaration(GlobalScope::Global, declaration)
            }
        }
    }

    pub(super) fn check_class_signature(
        &mut self,
        class: &ast::ClassSignature,
    ) -> Result<(), Error> {
        if let Some(supertype) = &class.extends {
            let symbol = self.check_variable(supertype)?;
            if let Some(span) = self.context.get_final(&symbol) {
                bail!(supertype.span, ErrorKind::FinalSuperclass(symbol, *span));
            }
        }

        for method in &class.methods {
            self.check_callable(method)?;
        }

        Ok(())
    }

    fn check_class(&mut self, class: &ast::Class) -> Result<(), Error> {
        if let Some(supertype) = &class.extends {
            let symbol = self.check_variable(supertype)?;
            if let Some(span) = self.context.get_final(&symbol) {
                bail!(supertype.span, ErrorKind::FinalSuperclass(symbol, *span));
            }
        }

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

        for item in &class.items {
            match item {
                ast::ClassItem::Field(declaration) => {
                    self.check_declaration(GlobalScope::Class(class.name.symbol), declaration)?;
                }
                ast::ClassItem::Method(method) => {
                    // Check if method is declared or defined by an ancestor
                    if let Some((
                        span,
                        Entry::Signature(old_parameters, old_returns)
                        | Entry::Function(old_parameters, old_returns),
                    )) = self
                        .context
                        .ancestors_exclusive(&class.name.symbol)
                        .find_map(|class| self.context.get_class(&class).unwrap().get(&method.name))
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

                    self.check_function(GlobalScope::Class(class.name.symbol), method)?
                }
            }
        }

        Ok(())
    }

    fn check_function(
        &mut self,
        scope: GlobalScope,
        function: &ast::Function,
    ) -> Result<(), Error> {
        self.check_callable(function)?;

        let returns = match self.context.get(scope, &function.name) {
            Some(Entry::Function(_, returns)) => returns.clone(),
            _ => panic!("[INTERNAL ERROR]: functions and methods should be bound in first pass"),
        };

        let is_procedure = returns.is_empty();
        let scope = match scope {
            GlobalScope::Class(class) => LocalScope::Method { class, returns },
            GlobalScope::Global => LocalScope::Function { returns },
        };

        self.context.push(scope);

        for parameter in &function.parameters {
            self.check_single_declaration(Scope::Local, parameter)?;
        }

        let r#type = self.check_statement(&function.statements)?;

        if r#type != r#type::Statement::Void && !is_procedure {
            bail!(function.span, ErrorKind::MissingReturn);
        }

        self.context.pop();

        Ok(())
    }

    pub(super) fn check_callable<C: ast::Callable>(&mut self, function: &C) -> Result<(), Error> {
        for parameter in function.parameters() {
            self.check_type(&parameter.r#type)?;
        }

        for r#return in function.returns() {
            self.check_type(r#return)?;
        }

        Ok(())
    }

    fn check_statement(&mut self, statement: &ast::Statement) -> Result<r#type::Statement, Error> {
        match statement {
            ast::Statement::Assignment(left, right, _) => {
                let left_span = left.span();
                let right_span = right.span();
                let left = self.check_expression(left)?;
                let right = self.check_expression(right)?;
                if self.context.is_subtype(&right, &left) {
                    Ok(r#type::Statement::Unit)
                } else {
                    expected!(left_span, left, right_span, right)
                }
            }
            ast::Statement::Call(call) => match &*self.check_call(call)? {
                [] => Ok(r#type::Statement::Unit),
                _ => bail!(call.span, ErrorKind::NotProcedure),
            },
            ast::Statement::Declaration(declaration, _) => {
                self.check_declaration(Scope::Local, declaration)?;
                Ok(r#type::Statement::Unit)
            }
            ast::Statement::Initialization(initialization) => {
                self.check_initialization(Scope::Local, initialization)?;
                Ok(r#type::Statement::Unit)
            }
            ast::Statement::Return(returns, span) => {
                let returns = returns
                    .iter()
                    .map(|r#return| self.check_expression(r#return))
                    .collect::<Result<Vec<_>, _>>()?;

                let expected = self
                    .context
                    .get_scoped_returns()
                    .expect("[INTERNAL PARSER ERROR]: return outside function scope");

                if self.context.all_subtype(&returns, expected) {
                    Ok(r#type::Statement::Void)
                } else {
                    bail!(*span, ErrorKind::ReturnMismatch);
                }
            }
            ast::Statement::Sequence(statements, _) => {
                self.context.push(LocalScope::Block);

                let mut r#type = r#type::Statement::Unit;

                for statement in statements {
                    if r#type == r#type::Statement::Void {
                        bail!(statement.span(), ErrorKind::Unreachable)
                    } else if self.check_statement(statement)? == r#type::Statement::Void {
                        r#type = r#type::Statement::Void;
                    }
                }

                self.context.pop();
                Ok(r#type)
            }
            ast::Statement::If(condition, r#if, None, _) => {
                match self.check_expression(condition)? {
                    r#type::Expression::Boolean => (),
                    r#type => expected!(r#type::Expression::Boolean, condition.span(), r#type),
                };

                self.context.push(LocalScope::If);
                self.check_statement(r#if)?;
                self.context.pop();

                Ok(r#type::Statement::Unit)
            }
            ast::Statement::If(condition, r#if, Some(r#else), _) => {
                match self.check_expression(condition)? {
                    r#type::Expression::Boolean => (),
                    r#type => expected!(r#type::Expression::Boolean, condition.span(), r#type),
                };

                self.context.push(LocalScope::If);
                let r#if = self.check_statement(r#if)?;
                self.context.pop();

                self.context.push(LocalScope::Else);
                let r#else = self.check_statement(r#else)?;
                self.context.pop();

                Ok(r#if.least_upper_bound(&r#else))
            }
            ast::Statement::While(_, condition, body, _) => {
                match self.check_expression(condition)? {
                    r#type::Expression::Boolean => (),
                    r#type => expected!(r#type::Expression::Boolean, condition.span(), r#type),
                };

                self.context.push(LocalScope::While(None));
                self.check_statement(body)?;
                self.context.pop();

                Ok(r#type::Statement::Unit)
            }
            ast::Statement::Break(span) => match self.context.get_scoped_while() {
                None => bail!(*span, ErrorKind::NotInWhile),
                Some(_) => Ok(r#type::Statement::Void),
            },
        }
    }

    fn check_expression(&self, expression: &ast::Expression) -> Result<r#type::Expression, Error> {
        match expression {
            ast::Expression::Boolean(_, _) => Ok(r#type::Expression::Boolean),
            ast::Expression::Character(_, _) => Ok(r#type::Expression::Integer),
            ast::Expression::String(_, _) => Ok(r#type::Expression::Array(Box::new(
                r#type::Expression::Integer,
            ))),
            ast::Expression::Integer(_, _) => Ok(r#type::Expression::Integer),
            ast::Expression::Null(_) => Ok(r#type::Expression::Null),
            ast::Expression::This(span) => match self.context.get_scoped_class() {
                None => bail!(*span, ErrorKind::NotInClass(None)),
                Some(class) => Ok(r#type::Expression::Class(class)),
            },
            ast::Expression::Super(span) => self
                .context
                .get_scoped_class()
                .ok_or(ErrorKind::NotInClass(None))
                .and_then(|class| {
                    self.context
                        .get_superclass(&class)
                        .ok_or(ErrorKind::NoSuperclass(class))
                })
                .map(r#type::Expression::Class)
                .map_err(|kind| Error::new(*span, kind))
                .map_err(Error::from),
            ast::Expression::Variable(ast::Variable {
                name,
                generics,
                span: _,
            }) => {
                assert!(generics.is_none());
                match self.context.get(Scope::Local, name) {
                    Some(Entry::Variable(r#type)) => Ok(r#type.clone()),
                    Some(_) => bail!(*name.span, ErrorKind::NotVariable(name.symbol)),
                    None => bail!(*name.span, ErrorKind::UnboundVariable(name.symbol)),
                }
            }

            ast::Expression::Array(array, _) => {
                let mut bound = r#type::Expression::Any;
                let mut bound_span = None;

                for expression in array {
                    let r#type = self.check_expression(expression)?;
                    match self.context.least_upper_bound(&bound, &r#type) {
                        None => expected!(bound_span.unwrap(), bound, expression.span(), r#type),
                        Some(LeastUpperBound::Left(_bound)) => bound = _bound,
                        Some(LeastUpperBound::Right(_bound)) => {
                            bound = _bound;
                            bound_span = Some(expression.span());
                        }
                    }
                }

                Ok(r#type::Expression::Array(Box::new(bound)))
            }

            ast::Expression::Binary(binary, left, right, _) => {
                let left_span = left.span();
                let right_span = right.span();

                let left = self.check_expression(left)?;
                let right = self.check_expression(right)?;

                let (parameter, r#return) = match binary.get() {
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
                        if self.context.is_subtype(&left, &right)
                            || self.context.is_subtype(&right, &left)
                        {
                            return Ok(r#type::Expression::Boolean);
                        } else {
                            expected!(left_span, left, right_span, right);
                        }
                    }
                };

                if let (
                    ast::Binary::Add | ast::Binary::Cat,
                    r#type::Expression::Array(_),
                    r#type::Expression::Array(_),
                ) = (binary.get(), &left, &right)
                {
                    return match self.context.least_upper_bound(&left, &right) {
                        None => expected!(left_span, left, right_span, right),
                        Some(LeastUpperBound::Left(r#type) | LeastUpperBound::Right(r#type)) => {
                            binary.set(ast::Binary::Cat);
                            Ok(r#type)
                        }
                    };
                }

                if self.context.is_subtype(&left, &parameter)
                    && self.context.is_subtype(&right, &parameter)
                {
                    Ok(r#return)
                } else if self.context.is_subtype(&left, &parameter) {
                    expected!(parameter, right_span, right)
                } else {
                    expected!(parameter, left_span, left)
                }
            }

            ast::Expression::Unary(ast::Unary::Neg, expression, _) => {
                match self.check_expression(expression)? {
                    r#type::Expression::Integer => Ok(r#type::Expression::Integer),
                    r#type => expected!(r#type::Expression::Integer, expression.span(), r#type),
                }
            }
            ast::Expression::Unary(ast::Unary::Not, expression, _) => {
                match self.check_expression(expression)? {
                    r#type::Expression::Boolean => Ok(r#type::Expression::Boolean),
                    r#type => expected!(r#type::Expression::Boolean, expression.span(), r#type),
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
            ast::Expression::Length(array, span) => match self.check_expression(array)? {
                r#type::Expression::Array(_) => Ok(r#type::Expression::Integer),
                r#type => expected!(
                    r#type::Expression::Array(Box::new(r#type::Expression::Any)),
                    *span,
                    r#type,
                ),
            },

            ast::Expression::Dot(receiver_class, receiver, field, _) => {
                let class = match self.check_expression(receiver)? {
                    r#type::Expression::Class(class) => class,
                    _ => bail!(receiver.span(), ErrorKind::NotClass),
                };

                receiver_class.set(Some(class));

                match self.context.get(GlobalScope::Class(class), field) {
                    None => bail!(*field.span, ErrorKind::UnboundVariable(field.symbol)),
                    Some(Entry::Variable(r#type)) => Ok(r#type.clone()),
                    Some(_) => bail!(*field.span, ErrorKind::NotVariable(field.symbol)),
                }
            }
            ast::Expression::New(
                ast::Variable {
                    name,
                    generics,
                    span: _,
                },
                span,
            ) => {
                assert!(generics.is_none());
                match self.context.get_class_implementation(name) {
                    Some(_) => Ok(r#type::Expression::Class(name.symbol)),
                    None if self.context.get_class(name).is_some() => {
                        bail!(*span, ErrorKind::NotInClassModule(name.symbol))
                    }
                    None => {
                        bail!(*name.span, ErrorKind::UnboundClass(name.symbol))
                    }
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
    ) -> Result<Vec<r#type::Expression>, Error> {
        match expression {
            ast::Expression::Call(call) => self.check_call(call),
            expression => Ok(vec![self.check_expression(expression)?]),
        }
    }

    fn check_call(&self, call: &ast::Call) -> Result<Vec<r#type::Expression>, Error> {
        let (scope, function_span, function_name) = match &*call.function {
            ast::Expression::Variable(ast::Variable {
                name,
                generics,
                span: _,
            }) => {
                assert!(generics.is_none());
                (Scope::Local, *name.span, name.symbol)
            }
            ast::Expression::Dot(receiver_class, receiver, name, span) => {
                let class = match self.check_expression(receiver)? {
                    r#type::Expression::Class(class) => class,
                    _ => bail!(receiver.span(), ErrorKind::NotClass),
                };

                receiver_class.set(Some(class));

                (Scope::Global(GlobalScope::Class(class)), *span, name.symbol)
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

        for (argument, parameter) in call.arguments.iter().zip(parameters) {
            let r#type = self.check_expression(argument)?;

            if !self.context.is_subtype(&r#type, parameter) {
                // TODO: attach span to parameters
                expected!(parameter.clone(), argument.span(), r#type)
            }
        }

        Ok(returns.clone())
    }

    fn check_initialization<S: Into<Scope>>(
        &mut self,
        scope: S,
        ast::Initialization {
            declarations,
            expression,
            span,
        }: &ast::Initialization,
    ) -> Result<(), Error> {
        let r#types = self.check_call_or_expression(expression)?;

        if r#types.is_empty() {
            bail!(*span, ErrorKind::InitProcedure);
        }

        if r#types.len() != declarations.len() {
            bail!(*span, ErrorKind::InitLength);
        }

        let scope = scope.into();
        for (declaration, subtype) in declarations
            .iter()
            .zip(r#types)
            .filter_map(|(declaration, r#type)| Some((declaration.as_ref()?, r#type)))
        {
            let supertype = self.check_single_declaration(scope, declaration)?;
            if !self.context.is_subtype(&subtype, &supertype) {
                expected!(declaration.span(), supertype, expression.span(), subtype);
            }
        }

        Ok(())
    }

    fn check_declaration<S: Into<Scope>>(
        &mut self,
        scope: S,
        declaration: &ast::Declaration,
    ) -> Result<(), Error> {
        let multiple = match declaration {
            ast::Declaration::Multiple(multiple) => multiple,
            ast::Declaration::Single(single) => {
                return self.check_single_declaration(scope, single).map(drop);
            }
        };

        let scope = scope.into();
        for name in &multiple.names {
            self.check_single_declaration(
                scope,
                &ast::SingleDeclaration {
                    name: name.clone(),
                    r#type: multiple.r#type.clone(),
                    span: multiple.span,
                },
            )?;
        }

        Ok(())
    }

    fn check_single_declaration<S: Into<Scope>>(
        &mut self,
        scope: S,
        ast::SingleDeclaration {
            name,
            r#type,
            span: _,
        }: &ast::SingleDeclaration,
    ) -> Result<r#type::Expression, Error> {
        let r#type = self.check_type(r#type)?;
        let scope = scope.into();

        match (
            scope,
            self.context
                .insert(scope, name.clone(), Entry::Variable(r#type.clone())),
        ) {
            (_, None) => (),
            // Note: class fields are inserted during loading, since they need
            // to be visible everywhere.
            (Scope::Global(GlobalScope::Class(_)), Some(_)) => (),
            (Scope::Local | Scope::Global(GlobalScope::Global), Some((span, _))) => {
                bail!(*name.span, ErrorKind::NameClash(span))
            }
        }

        Ok(r#type)
    }

    fn check_type(&self, r#type: &ast::Type) -> Result<r#type::Expression, Error> {
        match r#type {
            ast::Type::Bool(_) => Ok(r#type::Expression::Boolean),
            ast::Type::Int(_) => Ok(r#type::Expression::Integer),
            ast::Type::Array(r#type, None, _) => self
                .check_type(r#type)
                .map(Box::new)
                .map(r#type::Expression::Array),
            ast::Type::Class(variable) => {
                self.check_variable(variable).map(r#type::Expression::Class)
            }
            ast::Type::Array(r#type, Some(length), _) => {
                let r#type = self.check_type(r#type)?;
                match self.check_expression(length)? {
                    r#type::Expression::Integer => Ok(r#type::Expression::Array(Box::new(r#type))),
                    r#type => expected!(r#type::Expression::Integer, length.span(), r#type),
                }
            }
        }
    }

    // There are two cases where this function is called:
    // FIXME: three cases (call during monomorphization pass)
    //
    // 1) Checking interfaces during the second half of the loading pass.
    //
    // Here, we haven't monomorphized yet, so this branch is reachable,
    // and type arguments should be checked. But we *don't* want to check
    // that the template instantiation exists, because its implementation
    // may be in a separate compilation unit.
    //
    // 2) Checking signatures after monomorphization.
    //
    // Here, there are no more generics, so this branch is unreachable.
    // Any unbound classes within the type arguments must be caught
    // during the monomorphization pass.
    fn check_variable(&self, variable: &ast::Variable) -> Result<Symbol, Error> {
        match &variable.generics {
            None => match self.context.get_class(&variable.name) {
                Some(_) => Ok(variable.name.symbol),
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
                    .iter()
                    .map(|generic| self.check_type(generic))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(abi::mangle::template(&variable.name.symbol, &generics))
            }
        }
    }
}
