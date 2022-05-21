use std::io;
use std::path::Path;

use crate::check::check::Checker;
use crate::check::Entry;
use crate::check::Error;
use crate::check::ErrorKind;
use crate::check::GlobalScope;
use crate::data::ast;
use crate::data::r#type;
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
        match self.context.insert_class(class.name) {
            Some(_) => bail!(class.span, ErrorKind::NameClash),
            None => Ok(()),
        }
    }

    fn check_class_signature(&mut self, class: &ast::ClassSignature) -> Result<(), error::Error> {
        if let Some(supertype) = class.extends {
            if self.context.get_class(&supertype).is_none() {
                bail!(class.span, ErrorKind::UnboundClass(supertype))
            }

            assert!(self
                .context
                .insert_supertype(class.name, supertype)
                .is_none());

            if self.context.has_cycle(&class.name) {
                bail!(class.span, ErrorKind::ClassCycle(class.name));
            }
        }

        for method in &class.methods {
            self.check_function_signature(GlobalScope::Class(class.name), method)?;
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

        match self.context.get(scope, &function.name) {
            Some(existing) if *existing == signature => (),
            Some(_) => bail!(function.span, ErrorKind::NameClash),
            None => {
                self.context.insert(scope, function.name, signature);
            }
        }

        Ok(())
    }

    pub(super) fn load_class(&mut self, class: &ast::Class) -> Result<(), error::Error> {
        // May already be defined by an interface
        if self.context.get_class(&class.name).is_none() {
            self.context.insert_class(class.name);
        }

        for item in &class.items {
            match item {
                // Note: relies on the assumption that fields can have neither length expressions
                // in array types, nor initializer expressions, so they can be checked linearly.
                ast::ClassItem::Field(_) => (),
                ast::ClassItem::Method(method) => {
                    self.load_function(GlobalScope::Class(class.name), method)?
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
            ast::Type::Class(class, _) => Ok(r#type::Expression::Class(*class)),
            ast::Type::Array(r#type, _, _) => self
                .load_type(r#type)
                .map(Box::new)
                .map(r#type::Expression::Array),
        }
    }
}