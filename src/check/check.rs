use std::fs;
use std::path::Path;

use crate::check::context::Entry;
use crate::check::context::GlobalScope;
use crate::check::context::LocalScope;
use crate::check::Context;
use crate::check::Error;
use crate::check::ErrorKind;
use crate::check::Scope;
use crate::data::ast;
use crate::data::r#type;
use crate::data::symbol;
use crate::error;
use crate::lex;
use crate::parse;

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

pub fn check(library: &Path, path: &Path, program: &ast::Program) -> Result<Context, crate::Error> {
    Checker::new().check_program(library, path, program)
}

struct Checker {
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
        directory_library: &Path,
        _path: &Path,
        program: &ast::Program,
    ) -> Result<Context, error::Error> {
        for path in &program.uses {
            let path = directory_library.join(symbol::resolve(path.name).to_string() + ".ixi");
            let source = fs::read_to_string(path)?;
            let lexer = lex::Lexer::new(&source);
            let interface = parse::InterfaceParser::new().parse(lexer)?;
            self.load_interface(&interface)?;
        }

        for item in &program.items {
            match item {
                ast::Item::Global(_) => todo!(),
                ast::Item::Class(_) => todo!(),
                ast::Item::Function(function) => self.load_function(function)?,
            }
        }

        for item in &program.items {
            match item {
                ast::Item::Global(ast::Global::Declaration(declaration)) => {
                    self.check_declaration(GlobalScope::Global, declaration)?;
                }
                ast::Item::Global(ast::Global::Initialization(initialization)) => {
                    self.check_initialization(GlobalScope::Global, initialization)?;
                }
                ast::Item::Class(class) => self.check_class(class)?,
                ast::Item::Function(function) => {
                    let returns = match self.context.get(GlobalScope::Global, &function.name) {
                        Some(Entry::Function(_, returns)) => returns.clone(),
                        _ => panic!("[INTERNAL ERROR]: function should be bound in first pass"),
                    };

                    self.check_function(
                        LocalScope::Function {
                            returns: returns.clone(),
                        },
                        function,
                    )?;
                }
            }
        }

        Ok(self.context)
    }

    fn load_interface(&mut self, interface: &ast::Interface) -> Result<(), error::Error> {
        for item in &interface.items {
            match item {
                ast::ItemSignature::Class(class) => self.load_class_signature(class)?,
                ast::ItemSignature::Function(function) => {
                    self.load_function_signature(GlobalScope::Global, function)?;
                }
            };
        }
        Ok(())
    }

    fn load_class_signature(&mut self, class: &ast::ClassSignature) -> Result<(), error::Error> {
        if let Some(supertype) = class.extends {
            if let Some(existing) = self.context.insert_subtype(class.name, supertype) {
                expected!(
                    class.span,
                    r#type::Expression::Class(existing),
                    r#type::Expression::Class(supertype)
                );
            }
        }

        for method in &class.methods {
            self.load_function_signature(GlobalScope::Class(class.name), method)?;
        }

        Ok(())
    }

    fn load_function_signature(
        &mut self,
        scope: GlobalScope,
        function: &ast::FunctionSignature,
    ) -> Result<(), error::Error> {
        let (parameters, returns) = self.check_signature(function)?;
        let signature = Entry::Signature(parameters, returns);

        match self.context.get(scope, &function.name) {
            Some(existing) if *existing == signature => (),
            Some(_) => bail!(function.span, ErrorKind::NameClash),
            None => {
                self.context.insert(scope, function.name, signature);
            }
        }

        Ok(())
    }

    fn load_function(&mut self, function: &ast::Function) -> Result<(), error::Error> {
        let (new_parameters, new_returns) = self.check_signature(function)?;

        match self.context.get(GlobalScope::Global, &function.name) {
            Some(Entry::Signature(old_parameters, _))
                if !self.context.all_subtype(old_parameters, &new_parameters) =>
            {
                bail!(function.span, ErrorKind::SignatureMismatch)
            }
            Some(Entry::Signature(_, old_returns))
                if !self.context.all_subtype(&new_returns, old_returns) =>
            {
                bail!(function.span, ErrorKind::SignatureMismatch)
            }
            Some(Entry::Signature(_, _)) | None => {
                self.context.insert(
                    GlobalScope::Global,
                    function.name,
                    Entry::Function(new_parameters, new_returns),
                );
            }
            Some(Entry::Function(_, _)) | Some(Entry::Variable(_)) => {
                bail!(function.span, ErrorKind::NameClash)
            }
        }

        Ok(())
    }

    fn check_signature<C: ast::Callable>(
        &self,
        signature: &C,
    ) -> Result<(Vec<r#type::Expression>, Vec<r#type::Expression>), error::Error> {
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

        Ok((parameters, returns))
    }

    fn check_type(&self, r#type: &ast::Type) -> Result<r#type::Expression, error::Error> {
        match r#type {
            ast::Type::Bool(_) => Ok(r#type::Expression::Boolean),
            ast::Type::Int(_) => Ok(r#type::Expression::Integer),
            ast::Type::Class(class, _) => Ok(r#type::Expression::Class(*class)),
            ast::Type::Array(r#type, None, _) => self
                .check_type(r#type)
                .map(Box::new)
                .map(r#type::Expression::Array),
            ast::Type::Array(r#type, Some(length), _) => {
                let r#type = self.check_type(r#type)?;
                match self.check_expression(length)? {
                    r#type::Expression::Integer => Ok(r#type::Expression::Array(Box::new(r#type))),
                    r#type => expected!(length.span(), r#type::Expression::Integer, r#type),
                }
            }
        }
    }

    fn check_class(&mut self, class: &ast::Class) -> Result<(), error::Error> {
        if let Some(supertype) = class.extends {
            if let Some(existing) = self.context.insert_subtype(class.name, supertype) {
                expected!(
                    class.span,
                    r#type::Expression::Class(existing),
                    r#type::Expression::Class(supertype)
                );
            }
        }

        for item in &class.items {
            match item {
                ast::ClassItem::Field(declaration) => {
                    self.check_declaration(GlobalScope::Class(class.name), declaration)?;
                }
                ast::ClassItem::Method(method) => {
                    let returns = match self
                        .context
                        .get(GlobalScope::Class(class.name), &method.name)
                    {
                        Some(Entry::Function(_, returns)) => returns.clone(),
                        _ => panic!("[INTERNAL ERROR]: function should be bound in first pass"),
                    };

                    self.check_function(
                        LocalScope::Method {
                            class: class.name,
                            returns: returns.clone(),
                        },
                        method,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn check_function(
        &mut self,
        scope: LocalScope,
        function: &ast::Function,
    ) -> Result<(), error::Error> {
        self.context.push(scope.clone());

        for parameter in &function.parameters {
            self.check_single_declaration(Scope::Local, parameter)?;
        }

        if self.check_statement(&function.statements)? != r#type::Statement::Void
            && !scope.returns().unwrap().is_empty()
        {
            bail!(function.span, ErrorKind::MissingReturn);
        }

        self.context.pop();
        Ok(())
    }

    fn check_call(&self, call: &ast::Call) -> Result<Vec<r#type::Expression>, error::Error> {
        let name = match &*call.function {
            ast::Expression::Variable(name, _) => *name,
            _ => todo!(),
        };

        let (parameters, returns) = match self.context.get(Scope::Local, &name) {
            Some(Entry::Signature(parameters, returns))
            | Some(Entry::Function(parameters, returns)) => (parameters, returns),
            Some(_) => bail!(call.span, ErrorKind::NotFun(name)),
            None => bail!(call.span, ErrorKind::UnboundFun(name)),
        };

        if call.arguments.len() != parameters.len() {
            bail!(call.span, ErrorKind::CallLength);
        }

        for (argument, parameter) in call.arguments.iter().zip(parameters) {
            let r#type = self.check_expression(argument)?;

            if !self.context.is_subtype(&r#type, parameter) {
                expected!(argument.span(), parameter.clone(), r#type)
            }
        }

        Ok(returns.clone())
    }

    fn check_declaration<S: Into<Scope>>(
        &mut self,
        scope: S,
        declaration: &ast::Declaration,
    ) -> Result<(), error::Error> {
        match declaration {
            ast::Declaration::Multiple(multiple) => {
                let scope = scope.into();
                for name in &multiple.names {
                    self.check_single_declaration(
                        scope,
                        &ast::SingleDeclaration {
                            name: *name,
                            r#type: multiple.r#type.clone(),
                            span: multiple.span,
                        },
                    )?;
                }
            }
            ast::Declaration::Single(single) => {
                self.check_single_declaration(scope, single)?;
            }
        }
        Ok(())
    }

    fn check_single_declaration<S: Into<Scope>>(
        &mut self,
        scope: S,
        declaration: &ast::SingleDeclaration,
    ) -> Result<r#type::Expression, error::Error> {
        let r#type = self.check_type(&declaration.r#type)?;

        if self
            .context
            .insert(scope, declaration.name, Entry::Variable(r#type.clone()))
            .is_some()
        {
            bail!(declaration.span, ErrorKind::NameClash);
        }

        Ok(r#type)
    }

    fn check_expression(
        &self,
        expression: &ast::Expression,
    ) -> Result<r#type::Expression, error::Error> {
        match expression {
            ast::Expression::Boolean(_, _) => Ok(r#type::Expression::Boolean),
            ast::Expression::Character(_, _) => Ok(r#type::Expression::Integer),
            ast::Expression::String(_, _) => Ok(r#type::Expression::Array(Box::new(
                r#type::Expression::Integer,
            ))),
            ast::Expression::Integer(_, _) => Ok(r#type::Expression::Integer),
            ast::Expression::This(_) => todo!(),
            ast::Expression::Null(_) => todo!(),
            ast::Expression::Variable(name, span) => match self.context.get(Scope::Local, name) {
                Some(Entry::Variable(typ)) => Ok(typ.clone()),
                Some(_) => bail!(*span, ErrorKind::NotVariable(*name)),
                None => bail!(*span, ErrorKind::UnboundVariable(*name)),
            },

            ast::Expression::Array(array, _) => {
                let mut bound = r#type::Expression::Any;

                for expression in array {
                    let r#type = self.check_expression(expression)?;
                    match self.context.least_upper_bound(&r#type, &bound) {
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
                    ast::Binary::Lt | ast::Binary::Le | ast::Binary::Ge | ast::Binary::Gt => {
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
                    ast::Binary::Ne | ast::Binary::Eq => {
                        let span = right.span();
                        let left = self.check_expression(left)?;
                        let right = self.check_expression(right)?;
                        if self.context.is_subtype(&left, &right)
                            || self.context.is_subtype(&right, &left)
                        {
                            return Ok(r#type::Expression::Boolean);
                        } else {
                            expected!(span, left, right);
                        }
                    }
                }

                let span = right.span();

                if let (r#type::Expression::Array(left), r#type::Expression::Array(right)) =
                    (self.check_expression(left)?, self.check_expression(right)?)
                {
                    return match self.context.least_upper_bound(&left, &right) {
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
            ast::Expression::Length(array, span) => match self.check_expression(array)? {
                r#type::Expression::Array(_) => Ok(r#type::Expression::Integer),
                typ => expected!(
                    *span,
                    r#type::Expression::Array(Box::new(r#type::Expression::Any)),
                    typ
                ),
            },

            ast::Expression::Dot(_, _, _) => todo!(),
            ast::Expression::New(_, _) => todo!(),

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
            ast::Expression::Call(call) => self.check_call(call),
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
            (left, right)
                if self.context.is_subtype(&left, &parameter)
                    && self.context.is_subtype(&right, &parameter) =>
            {
                Ok(r#return)
            }
            (left, mismatch) if self.context.is_subtype(&left, &parameter) => {
                expected!(right.span(), parameter, mismatch)
            }
            (mismatch, _) => expected!(left.span(), parameter, mismatch),
        }
    }

    fn check_statement(
        &mut self,
        statement: &ast::Statement,
    ) -> Result<r#type::Statement, error::Error> {
        match statement {
            ast::Statement::Assignment(left, right, _) => {
                let span = right.span();
                let left = self.check_expression(left)?;
                let right = self.check_expression(right)?;
                if self.context.is_subtype(&right, &left) {
                    Ok(r#type::Statement::Unit)
                } else {
                    expected!(span, left, right)
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
                    .get_returns()
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
                    typ => expected!(condition.span(), r#type::Expression::Boolean, typ),
                };

                self.context.push(LocalScope::If);
                self.check_statement(r#if)?;
                self.context.pop();

                Ok(r#type::Statement::Unit)
            }
            ast::Statement::If(condition, r#if, Some(r#else), _) => {
                match self.check_expression(condition)? {
                    r#type::Expression::Boolean => (),
                    typ => expected!(condition.span(), r#type::Expression::Boolean, typ),
                };

                self.context.push(LocalScope::If);
                let r#if = self.check_statement(r#if)?;
                self.context.pop();

                self.context.push(LocalScope::Else);
                let r#else = self.check_statement(r#else)?;
                self.context.pop();

                Ok(r#if.least_upper_bound(&r#else))
            }
            ast::Statement::While(_, cond, body, _) => {
                match self.check_expression(cond)? {
                    r#type::Expression::Boolean => (),
                    typ => expected!(cond.span(), r#type::Expression::Boolean, typ),
                };

                self.context.push(LocalScope::While);
                self.check_statement(body)?;
                self.context.pop();

                Ok(r#type::Statement::Unit)
            }
            ast::Statement::Break(_) => todo!(),
        }
    }

    fn check_initialization<S: Into<Scope>>(
        &mut self,
        scope: S,
        ast::Initialization {
            declarations,
            expression,
            span,
        }: &ast::Initialization,
    ) -> Result<(), error::Error> {
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
                expected!(declaration.span, subtype, supertype);
            }
        }

        Ok(())
    }
}
