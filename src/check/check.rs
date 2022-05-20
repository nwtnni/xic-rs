use std::io;
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
use crate::data::span::Span;
use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::error;
use crate::lex;
use crate::parse;
use crate::Set;

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

pub fn check(
    directory_library: Option<&Path>,
    path: &Path,
    program: &ast::Program,
) -> Result<Context, crate::Error> {
    Checker::new().check_program(directory_library, path, program)
}

struct Checker {
    context: Context,
    phase: Phase,

    /// Set of unique interfaces in the use tree
    used: Set<Symbol>,

    /// Set of classes defined in this module
    defined: Set<Symbol>,
}

// Because Xi allows functions, methods, and globals to reference symbols defined later
// in the program, we need to perform two passes: `Load`, to bind all globally visible
// names to their explicitly annotated types, and `Check`, to validate the entire program.
//
// This state variable is a bit dirty, but it allows us to inspect the phase only in methods
// that depend on it, rather than threading it through multiple layers of recursion.
#[derive(Copy, Clone, Debug)]
enum Phase {
    Load,
    Check,
}

impl Checker {
    fn new() -> Self {
        Checker {
            phase: Phase::Load,
            context: Context::new(),
            used: Set::default(),
            defined: Set::default(),
        }
    }

    pub fn check_program(
        mut self,
        directory_library: Option<&Path>,
        path: &Path,
        program: &ast::Program,
    ) -> Result<Context, error::Error> {
        let directory_library = directory_library.unwrap_or_else(|| path.parent().unwrap());

        for r#use in &program.uses {
            self.load_use(directory_library, r#use)?;
        }

        let implicit = path
            .file_name()
            .map(Path::new)
            .map(|path| ast::Use {
                name: symbol::intern(path.to_str().unwrap()),
                span: Span::default(),
            })
            .unwrap();

        match self.load_use(directory_library, &implicit) {
            Ok(()) => (),
            Err(error::Error::Semantic(error))
                if *error.kind() == ErrorKind::NotFound(implicit.name) => {}
            Err(error) => return Err(error),
        }

        for _ in 0..2 {
            for item in &program.items {
                match item {
                    ast::Item::Global(global) => self.check_global(global)?,
                    ast::Item::Class(class) => self.check_class(class)?,
                    ast::Item::Function(function) => self.check_function(None, function)?,
                }
            }

            self.phase = Phase::Check;
        }

        Ok(self.context)
    }

    fn load_use(&mut self, directory_library: &Path, r#use: &ast::Use) -> Result<(), error::Error> {
        if !self.used.insert(r#use.name) {
            return Ok(());
        }

        let path = directory_library.join(format!("{}.ixi", r#use.name));
        let tokens = match lex::lex(&path) {
            Ok(tokens) => tokens,
            Err(error::Error::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
                bail!(r#use.span, ErrorKind::NotFound(r#use.name))
            }
            Err(error) => return Err(error),
        };

        let interface = parse::InterfaceParser::new().parse(tokens)?;
        self.load_interface(directory_library, &interface)
    }

    fn load_interface(
        &mut self,
        directory_library: &Path,
        interface: &ast::Interface,
    ) -> Result<(), error::Error> {
        for r#use in &interface.uses {
            self.load_use(directory_library, r#use)?;
        }

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
        if self.context.insert_class(class.name).is_some() {
            bail!(class.span, ErrorKind::NameClash);
        }

        if let Some(supertype) = class.extends {
            assert!(self
                .context
                .insert_supertype(class.name, supertype)
                .is_none());

            if self.context.get_class(&supertype).is_none() {
                bail!(class.span, ErrorKind::UnboundClass(supertype))
            }

            if self.context.has_cycle(&class.name) {
                bail!(class.span, ErrorKind::ClassCycle(class.name));
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
        match (self.phase, r#type) {
            (_, ast::Type::Bool(_)) => Ok(r#type::Expression::Boolean),
            (_, ast::Type::Int(_)) => Ok(r#type::Expression::Integer),

            // Delay checking class existence
            (Phase::Load, ast::Type::Class(class, _)) => Ok(r#type::Expression::Class(*class)),
            (Phase::Check, ast::Type::Class(class, span)) => match self.context.get_class(class) {
                Some(_) => Ok(r#type::Expression::Class(*class)),
                None => bail!(*span, ErrorKind::UnboundClass(*class)),
            },

            // Delay checking length expression
            (_, ast::Type::Array(r#type, None, _))
            | (Phase::Load, ast::Type::Array(r#type, Some(_), _)) => self
                .check_type(r#type)
                .map(Box::new)
                .map(r#type::Expression::Array),

            (Phase::Check, ast::Type::Array(r#type, Some(length), _)) => {
                let r#type = self.check_type(r#type)?;
                match self.check_expression(length)? {
                    r#type::Expression::Integer => Ok(r#type::Expression::Array(Box::new(r#type))),
                    r#type => expected!(length.span(), r#type::Expression::Integer, r#type),
                }
            }
        }
    }

    fn check_global(&mut self, global: &ast::Global) -> Result<(), error::Error> {
        match global {
            ast::Global::Declaration(declaration) => {
                self.check_declaration(GlobalScope::Global, declaration)
            }
            ast::Global::Initialization(initialization) => {
                self.check_initialization(GlobalScope::Global, initialization)
            }
        }
    }

    fn check_class(&mut self, class: &ast::Class) -> Result<(), error::Error> {
        match self.phase {
            Phase::Load => {
                if !self.defined.insert(class.name) {
                    bail!(class.span, ErrorKind::NameClash);
                }

                // May already be defined by an interface
                if self.context.get_class(&class.name).is_none() {
                    self.context.insert_class(class.name);
                }
            }
            Phase::Check => {
                if let Some(supertype) = class.extends {
                    if self.context.get_class(&supertype).is_none() {
                        bail!(class.span, ErrorKind::UnboundClass(supertype));
                    }

                    if let Some(existing) = self.context.insert_supertype(class.name, supertype) {
                        expected!(
                            class.span,
                            r#type::Expression::Class(existing),
                            r#type::Expression::Class(supertype)
                        );
                    }

                    if self.context.has_cycle(&class.name) {
                        bail!(class.span, ErrorKind::ClassCycle(class.name));
                    }
                }
            }
        }

        for item in &class.items {
            match item {
                ast::ClassItem::Field(declaration) => {
                    self.check_declaration(GlobalScope::Class(class.name), declaration)?;
                }
                ast::ClassItem::Method(method) => self.check_function(Some(class.name), method)?,
            }
        }

        Ok(())
    }

    fn check_function(
        &mut self,
        class: Option<Symbol>,
        function: &ast::Function,
    ) -> Result<(), error::Error> {
        let scope = class.map(GlobalScope::Class).unwrap_or(GlobalScope::Global);

        match self.phase {
            Phase::Load => {
                let (new_parameters, new_returns) = self.check_signature(function)?;

                match self.context.get(scope, &function.name) {
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
                            scope,
                            function.name,
                            Entry::Function(new_parameters, new_returns),
                        );
                    }
                    Some(Entry::Function(_, _)) | Some(Entry::Variable(_)) => {
                        bail!(function.span, ErrorKind::NameClash)
                    }
                }
            }
            Phase::Check => {
                let returns = match self.context.get(scope, &function.name) {
                    Some(Entry::Function(_, returns)) => returns.clone(),
                    _ => panic!(
                        "[INTERNAL ERROR]: functions and methods should be bound in first pass"
                    ),
                };

                let procedure = returns.is_empty();
                let scope = match class {
                    Some(class) => LocalScope::Method { class, returns },
                    None => LocalScope::Function { returns },
                };

                self.context.push(scope);

                for parameter in &function.parameters {
                    self.check_single_declaration(Scope::Local, parameter)?;
                }

                let r#type = self.check_statement(&function.statements)?;

                if r#type != r#type::Statement::Void && !procedure {
                    bail!(function.span, ErrorKind::MissingReturn);
                }

                self.context.pop();
            }
        }
        Ok(())
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
            ast::Expression::This(span) | ast::Expression::Null(span) => {
                match self.context.get_scoped_class() {
                    None => bail!(*span, ErrorKind::NotInClass(None)),
                    Some(class) => Ok(r#type::Expression::Class(class)),
                }
            }
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
                        let left_span = left.span();
                        let right_span = right.span();

                        let left = self.check_expression(left)?;
                        let right = self.check_expression(right)?;
                        let class = self.context.get_scoped_class();

                        match (&left, &right) {
                            (r#type::Expression::Class(left), r#type::Expression::Class(right))
                                if class != Some(*left) && class != Some(*right) =>
                            {
                                bail!(left_span, ErrorKind::NotInClass(Some(*left)));
                            }
                            (r#type::Expression::Class(left), _) if class != Some(*left) => {
                                bail!(left_span, ErrorKind::NotInClass(Some(*left)));
                            }
                            (_, r#type::Expression::Class(right)) if class != Some(*right) => {
                                bail!(right_span, ErrorKind::NotInClass(Some(*right)));
                            }
                            (_, _) => (),
                        }

                        if self.context.is_subtype(&left, &right)
                            || self.context.is_subtype(&right, &left)
                        {
                            return Ok(r#type::Expression::Boolean);
                        } else {
                            expected!(right_span, left, right);
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

            ast::Expression::Dot(receiver, field, span) => {
                let class = match self.check_expression(receiver)? {
                    r#type::Expression::Class(class) => class,
                    _ => bail!(*span, ErrorKind::NotClass),
                };

                match self.context.get(GlobalScope::Class(class), field) {
                    None => bail!(*span, ErrorKind::UnboundVariable(*field)),
                    Some(Entry::Variable(r#type)) => Ok(r#type.clone()),
                    Some(_) => bail!(*span, ErrorKind::NotVariable(*field)),
                }
            }
            ast::Expression::New(class, span) => match self.defined.contains(class) {
                false if self.context.get_class(class).is_some() => {
                    bail!(*span, ErrorKind::NotInClassModule(*class))
                }
                false => {
                    bail!(*span, ErrorKind::UnboundClass(*class))
                }
                true => Ok(r#type::Expression::Class(*class)),
            },

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

    fn check_call(&self, call: &ast::Call) -> Result<Vec<r#type::Expression>, error::Error> {
        let (scope, name) = match &*call.function {
            ast::Expression::Variable(name, _) => (Scope::Local, *name),
            ast::Expression::Dot(receiver, name, span) => {
                let class = match self.check_expression(receiver)? {
                    r#type::Expression::Class(class) => class,
                    _ => bail!(*span, ErrorKind::NotClass),
                };
                (Scope::Global(GlobalScope::Class(class)), *name)
            }
            expression => bail!(expression.span(), ErrorKind::NotFun(None)),
        };

        let (parameters, returns) = match self.context.get(scope, &name) {
            Some(Entry::Signature(parameters, returns))
            | Some(Entry::Function(parameters, returns)) => (parameters, returns),
            Some(_) => bail!(call.span, ErrorKind::NotFun(Some(name))),
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

    fn check_initialization<S: Into<Scope>>(
        &mut self,
        scope: S,
        ast::Initialization {
            declarations,
            expression,
            span,
        }: &ast::Initialization,
    ) -> Result<(), error::Error> {
        match self.phase {
            Phase::Check => (),

            // When loading, delay type-checking of the initializer expression
            Phase::Load => {
                let scope = scope.into();
                for declaration in declarations.iter().flatten() {
                    self.check_single_declaration(scope, declaration)?;
                }
                return Ok(());
            }
        }

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
                expected!(declaration.span, supertype, subtype);
            }
        }

        Ok(())
    }

    fn check_declaration<S: Into<Scope>>(
        &mut self,
        scope: S,
        declaration: &ast::Declaration,
    ) -> Result<(), error::Error> {
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
                    name: *name,
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
        ast::SingleDeclaration { name, r#type, span }: &ast::SingleDeclaration,
    ) -> Result<r#type::Expression, error::Error> {
        let scope = scope.into();
        let r#type = self.check_type(r#type)?;

        match (self.phase, scope) {
            // Should only be loading globally visible symbols
            (Phase::Load, Scope::Local) => unreachable!(),

            // Globals should only be mutually recursive with classes and functions,
            // but not other globals, because their initializers run in text order.
            (Phase::Load, Scope::Global(GlobalScope::Global)) => (),

            (Phase::Check, Scope::Local)
            | (Phase::Load, Scope::Global(GlobalScope::Class(_)))
            | (Phase::Check, Scope::Global(GlobalScope::Global)) => {
                if self
                    .context
                    .insert(scope, *name, Entry::Variable(r#type.clone()))
                    .is_some()
                {
                    bail!(*span, ErrorKind::NameClash)
                }
            }

            (Phase::Check, Scope::Global(GlobalScope::Class(_))) => {
                assert_eq!(
                    self.context.get(scope, name),
                    Some(&Entry::Variable(r#type.clone()))
                );
            }
        }

        Ok(r#type)
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
}
