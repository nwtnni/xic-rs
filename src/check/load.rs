use std::io;
use std::path::Path;

use crate::check::check::Checker;
use crate::check::Entry;
use crate::check::Error;
use crate::check::ErrorKind;
use crate::check::GlobalScope;
use crate::data::ast;
use crate::data::r#type;
use crate::data::span::Span;
use crate::error;
use crate::lex;
use crate::parse;

impl Checker {
    pub(super) fn load_use(
        &mut self,
        directory_library: &Path,
        r#use: &ast::Use,
    ) -> Result<(), error::Error> {
        // Load each interface exactly once
        if !self.used.insert(r#use.name.symbol) {
            return Ok(());
        }

        let path = directory_library.join(format!("{}.ixi", r#use.name));
        let tokens = match lex::lex(&path) {
            Ok(tokens) => tokens,
            Err(error::Error::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
                bail!(*r#use.name.span, ErrorKind::NotFound(r#use.name.symbol))
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
                ast::ItemSignature::Function(_) => (),
            }
        }

        for item in &interface.items {
            match item {
                ast::ItemSignature::Class(class) => self.check_class_signature(class)?,
                ast::ItemSignature::Function(function) => {
                    self.check_function_signature(GlobalScope::Global, function)?;
                }
            }
        }

        Ok(())
    }

    fn load_class_signature(&mut self, class: &ast::ClassSignature) -> Result<(), error::Error> {
        self.class_signatures.insert(class.name.clone());

        if self.context.get_class(&class.name.symbol).is_none() {
            self.context.insert_class(class.name.clone());
        }

        Ok(())
    }

    fn check_class_signature(&mut self, class: &ast::ClassSignature) -> Result<(), error::Error> {
        if let Some(supertype) = &class.extends {
            if self.context.get_class(&supertype.symbol).is_none() {
                bail!(*supertype.span, ErrorKind::UnboundClass(supertype.symbol))
            }

            if let Some(existing) = self
                .context
                .insert_supertype(class.name.clone(), supertype.clone())
            {
                expected!(
                    *existing.span,
                    r#type::Expression::Class(existing.symbol),
                    *supertype.span,
                    r#type::Expression::Class(supertype.symbol),
                );
            }

            if self.context.has_cycle(&class.name) {
                bail!(*class.name.span, ErrorKind::ClassCycle(class.name.symbol));
            }
        }

        for method in &class.methods {
            self.check_function_signature(GlobalScope::Class(class.name.symbol), method)?;
        }

        Ok(())
    }

    fn check_function_signature(
        &mut self,
        scope: GlobalScope,
        function: &ast::FunctionSignature,
    ) -> Result<(), error::Error> {
        let (parameters, returns) = self.load_callable(function)?;
        let signature = Entry::Signature(parameters, returns);

        match self.context.get_full(scope, &function.name.symbol) {
            Some((span, existing)) if *existing != signature => {
                bail!(*function.name.span, ErrorKind::NameClash(*span))
            }
            Some(_) | None => {
                self.context.insert_full(scope, &function.name, signature);
            }
        }

        Ok(())
    }

    pub(super) fn load_class(&mut self, class: &ast::Class) -> Result<(), error::Error> {
        if let Some(existing) = self.class_implementations.replace(class.name.clone()) {
            bail!(class.span, ErrorKind::NameClash(*existing.span));
        }

        // If not already declared by an interface
        if self.context.get_class(&class.name.symbol).is_none() {
            self.context.insert_class(class.name.clone());
        }

        if let Some(supertype) = &class.extends {
            if let Some(existing) = self
                .context
                .insert_supertype(class.name.clone(), supertype.clone())
            {
                expected!(
                    *existing.span,
                    r#type::Expression::Class(existing.symbol),
                    *supertype.span,
                    r#type::Expression::Class(supertype.symbol),
                );
            }

            if self.context.has_cycle(&class.name) {
                bail!(*class.name.span, ErrorKind::ClassCycle(class.name.symbol));
            }
        }

        for item in &class.items {
            match item {
                // Note: relies on the assumption that fields can have neither length expressions
                // in array types, nor initializer expressions, so they can be checked linearly.
                ast::ClassItem::Field(_) => (),
                ast::ClassItem::Method(method) => {
                    self.load_function(GlobalScope::Class(class.name.symbol), method)?
                }
            }
        }

        Ok(())
    }

    pub(super) fn load_function(
        &mut self,
        scope: GlobalScope,
        function: &ast::Function,
    ) -> Result<(), error::Error> {
        let (new_parameters, new_returns) = self.load_callable(function)?;

        match self.context.insert_full(
            scope,
            &function.name,
            Entry::Function(new_parameters.clone(), new_returns.clone()),
        ) {
            None => match scope {
                GlobalScope::Global => (),
                GlobalScope::Class(class) => match self.class_signatures.get(&ast::Identifier {
                    symbol: class,
                    span: Box::new(Span::default()),
                }) {
                    None => (),
                    Some(signature) => bail!(
                        *function.name.span,
                        ErrorKind::SignatureMismatch(*signature.span)
                    ),
                },
            },
            Some((span, Entry::Variable(_))) | Some((span, Entry::Function(_, _))) => {
                bail!(*function.name.span, ErrorKind::NameClash(span))
            }
            Some((span, Entry::Signature(old_parameters, old_returns))) => {
                if !self.context.all_subtype(&old_parameters, &new_parameters)
                    || !self.context.all_subtype(&new_returns, &old_returns)
                {
                    bail!(*function.name.span, ErrorKind::SignatureMismatch(span))
                }
            }
        }

        Ok(())
    }

    pub(super) fn load_callable<C: ast::Callable>(
        &self,
        signature: &C,
    ) -> Result<(Vec<r#type::Expression>, Vec<r#type::Expression>), error::Error> {
        let parameters = signature
            .parameters()
            .iter()
            .map(|declaration| &declaration.r#type)
            .map(|r#type| self.load_type(r#type))
            .collect::<Result<Vec<_>, _>>()?;

        let returns = signature
            .returns()
            .iter()
            .map(|r#type| self.load_type(r#type))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((parameters, returns))
    }

    fn load_type(&self, r#type: &ast::Type) -> Result<r#type::Expression, error::Error> {
        match r#type {
            ast::Type::Bool(_) => Ok(r#type::Expression::Boolean),
            ast::Type::Int(_) => Ok(r#type::Expression::Integer),
            ast::Type::Class(class) => Ok(r#type::Expression::Class(class.symbol)),
            ast::Type::Array(r#type, _, _) => self
                .load_type(r#type)
                .map(Box::new)
                .map(r#type::Expression::Array),
        }
    }
}
