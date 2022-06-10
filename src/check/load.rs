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
                ast::ItemSignature::ClassTemplate(_) => todo!(),
                ast::ItemSignature::Function(_) => (),
                ast::ItemSignature::FunctionTemplate(_) => todo!(),
            }
        }

        for item in &interface.items {
            match item {
                ast::ItemSignature::Class(class) => self.check_class_signature(class)?,
                ast::ItemSignature::ClassTemplate(_) => todo!(),
                ast::ItemSignature::Function(function) => {
                    self.check_function_signature(GlobalScope::Global, function)?;
                }
                ast::ItemSignature::FunctionTemplate(_) => todo!(),
            }
        }

        Ok(())
    }

    fn load_class_signature(&mut self, class: &ast::ClassSignature) -> Result<(), error::Error> {
        let expected = self.context.insert_class_signature(&class.name);

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

        for method in &class.methods {
            self.check_function_signature(GlobalScope::Class(class.name.symbol), method)?;
        }

        let (expected_span, expected) = match expected {
            Some(expected) => expected,
            None => return Ok(()),
        };

        let (actual_span, actual) = self.context.get_class(&class.name.symbol).unwrap();

        // New and old signatures must be exactly the same
        if expected.len() != actual.len()
            || expected.iter().any(|(symbol, (_, expected_entry))| {
                actual
                    .get(symbol)
                    .map_or(true, |(_, actual_entry)| expected_entry != actual_entry)
            })
            || actual.iter().any(|(symbol, (_, actual_entry))| {
                expected
                    .get(symbol)
                    .map_or(true, |(_, expected_entry)| actual_entry != expected_entry)
            })
        {
            bail!(*actual_span, ErrorKind::NameClash(expected_span));
        }

        Ok(())
    }

    fn check_class_signature(&mut self, class: &ast::ClassSignature) -> Result<(), error::Error> {
        if let Some(supertype) = &class.extends {
            if self.context.get_class(&supertype.symbol).is_none() {
                bail!(*supertype.span, ErrorKind::UnboundClass(supertype.symbol))
            }
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
        if let Some(span) = self.context.insert_class_implementation(&class.name) {
            bail!(class.span, ErrorKind::NameClash(span));
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
                ast::ClassItem::Field(field) => self.load_class_field(&class.name, field)?,
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
                GlobalScope::Class(class) => {
                    if let Some(span) = self.context.get_class_signature(&class) {
                        bail!(*function.name.span, ErrorKind::SignatureMismatch(*span));
                    }
                }
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

    fn load_class_field(
        &mut self,
        class: &ast::Identifier,
        declaration: &ast::Declaration,
    ) -> Result<(), error::Error> {
        for (name, r#type) in declaration.iter() {
            let r#type = self.load_type(r#type)?;
            if let Some((span, _)) = self.context.insert_full(
                GlobalScope::Class(class.symbol),
                name,
                Entry::Variable(r#type),
            ) {
                bail!(*name.span, ErrorKind::NameClash(span))
            }
        }

        Ok(())
    }

    fn load_type(&self, r#type: &ast::Type) -> Result<r#type::Expression, error::Error> {
        match r#type {
            ast::Type::Bool(_) => Ok(r#type::Expression::Boolean),
            ast::Type::Int(_) => Ok(r#type::Expression::Integer),
            ast::Type::Class(ast::Variable {
                name,
                generics: None,
                span: _,
            }) => Ok(r#type::Expression::Class(name.symbol)),
            ast::Type::Class(ast::Variable {
                name: _,
                generics: Some(_),
                span: _,
            }) => todo!(),
            ast::Type::Array(r#type, _, _) => self
                .load_type(r#type)
                .map(Box::new)
                .map(r#type::Expression::Array),
        }
    }
}
