use std::io;
use std::path::Path;

use crate::abi;
use crate::check::check::Checker;
use crate::check::Entry;
use crate::check::Error;
use crate::check::ErrorKind;
use crate::check::GlobalScope;
use crate::data::ast;
use crate::data::r#type;
use crate::data::span::Span;
use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::error;
use crate::lex;
use crate::parse;

impl Checker {
    pub(super) fn load_program(
        &mut self,
        directory_library: &Path,
        path: &Path,
        program: &ast::Program,
    ) -> Result<(), error::Error> {
        for r#use in &program.uses {
            self.load_use(directory_library, r#use)?;
        }

        let implicit = path
            .file_stem()
            .map(Path::new)
            .map(|path| ast::Use {
                name: ast::Identifier {
                    symbol: symbol::intern(path.to_str().unwrap()),
                    span: Box::new(Span::default()),
                },
                span: Span::default(),
            })
            .unwrap();

        match self.load_use(directory_library, &implicit) {
            Ok(()) => (),
            Err(error::Error::Semantic(error))
                if *error.kind() == ErrorKind::NotFound(implicit.name.symbol) => {}
            Err(error) => return Err(error),
        }

        for item in &program.items {
            match item {
                // Note: relies on the assumption that globals cannot have forward references
                // to other globals, since their initializers run in program order.
                ast::Item::Global(_) => (),
                ast::Item::Class(class) => self.load_class(class)?,
                ast::Item::ClassTemplate(class) => self.load_class_template(class)?,
                ast::Item::Function(function) => {
                    self.load_function(GlobalScope::Global, function)?
                }
                ast::Item::FunctionTemplate(function) => {
                    self.load_function_template(function)?;
                }
            }
        }

        Ok(())
    }

    fn load_use(&mut self, directory_library: &Path, r#use: &ast::Use) -> Result<(), error::Error> {
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
                ast::ItemSignature::ClassTemplate(class) => self.load_class_template(class)?,
                ast::ItemSignature::Function(function) => {
                    self.load_function_signature(GlobalScope::Global, function)?;
                }
                ast::ItemSignature::FunctionTemplate(function) => {
                    self.load_function_template(function)?;
                }
            }
        }

        for item in &interface.items {
            match item {
                ast::ItemSignature::Class(class) => self.check_class_signature(class)?,
                ast::ItemSignature::Function(function) => {
                    self.check_callable(function)?;
                }
                // TODO: can still implement some basic type-checking before instantiation
                // if we treat generics conservatively (i.e. as `Any` type)?
                ast::ItemSignature::ClassTemplate(_) => (),
                ast::ItemSignature::FunctionTemplate(_) => (),
            }
        }

        Ok(())
    }

    fn load_class_template(&mut self, class: &ast::ClassTemplate) -> Result<(), error::Error> {
        if let Some((span, _)) = self.context.insert_class_template(class.clone()) {
            bail!(class.span, ErrorKind::NameClash(span))
        }

        Ok(())
    }

    fn load_function_template(
        &mut self,
        function: &ast::FunctionTemplate,
    ) -> Result<(), error::Error> {
        if let Some((span, _)) = self.context.insert_function_template(function.clone()) {
            bail!(function.span, ErrorKind::NameClash(span))
        }

        Ok(())
    }

    fn load_class_signature(&mut self, class: &ast::ClassSignature) -> Result<(), error::Error> {
        if class.r#final {
            match self.context.insert_final(class.name.clone()) {
                Some(_) => (),
                None => {
                    if let Some((span, _)) = self.context.get_class_full(&class.name) {
                        bail!(*class.name.span, ErrorKind::FinalMismatch(*span));
                    }
                }
            }
        } else if let Some(span) = self.context.get_final(&class.name) {
            bail!(class.span, ErrorKind::FinalMismatch(*span));
        }

        let expected = self.context.insert_class_signature(class.name.clone());

        if let Some(supertype) = &class.extends {
            let symbol = self.load_variable(supertype);

            if let Some(existing) = self.context.insert_supertype(
                class.name.clone(),
                ast::Identifier {
                    symbol,
                    span: Box::new(supertype.span),
                },
            ) {
                expected!(
                    *existing.span,
                    r#type::Expression::Class(existing.symbol),
                    supertype.span,
                    r#type::Expression::Class(symbol),
                );
            }

            if self.context.has_cycle(&class.name) {
                bail!(*class.name.span, ErrorKind::ClassCycle(class.name.symbol));
            }
        }

        for method in &class.methods {
            self.load_function_signature(GlobalScope::Class(class.name.symbol), method)?;
        }

        let (expected_span, expected) = match expected {
            Some(expected) => expected,
            None => return Ok(()),
        };

        let (actual_span, actual) = self.context.get_class_full(&class.name).unwrap();

        // New and old signatures must be exactly the same
        if expected.len() != actual.len()
            || expected.iter().any(|(identifier, expected_entry)| {
                actual
                    .get(identifier)
                    .map_or(true, |(_, actual_entry)| expected_entry != actual_entry)
            })
            || actual.iter().any(|(identifier, actual_entry)| {
                expected
                    .get(identifier)
                    .map_or(true, |(_, expected_entry)| actual_entry != expected_entry)
            })
        {
            bail!(*actual_span, ErrorKind::NameClash(expected_span));
        }

        Ok(())
    }

    fn load_function_signature(
        &mut self,
        scope: GlobalScope,
        function: &ast::FunctionSignature,
    ) -> Result<(), error::Error> {
        let (parameters, returns) = self.load_callable(function);
        let signature = Entry::Signature(parameters, returns);

        match self.context.get_full(scope, &function.name.symbol) {
            Some((span, existing)) if *existing != signature => {
                bail!(*function.name.span, ErrorKind::NameClash(*span))
            }
            Some(_) | None => {
                self.context.insert(scope, function.name.clone(), signature);
            }
        }

        Ok(())
    }

    pub(super) fn load_class(&mut self, class: &ast::Class) -> Result<(), error::Error> {
        if class.r#final {
            match self.context.insert_final(class.name.clone()) {
                Some(_) => (),
                None => {
                    if let Some((span, _)) = self.context.get_class_full(&class.name) {
                        bail!(*class.name.span, ErrorKind::FinalMismatch(*span));
                    }
                }
            }
        } else if let Some(span) = self.context.get_final(&class.name) {
            bail!(class.span, ErrorKind::FinalMismatch(*span));
        }

        class
            .declared
            .set(self.context.get_class_signature(&class.name).is_some());

        if let Some(span) = self.context.insert_class_implementation(class.name.clone()) {
            bail!(class.span, ErrorKind::NameClash(span));
        }

        if let Some(supertype) = &class.extends {
            let symbol = self.load_variable(supertype);

            if let Some(existing) = self.context.insert_supertype(
                class.name.clone(),
                ast::Identifier {
                    symbol,
                    span: Box::new(supertype.span),
                },
            ) {
                expected!(
                    *existing.span,
                    r#type::Expression::Class(existing.symbol),
                    supertype.span,
                    r#type::Expression::Class(symbol),
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
        let (new_parameters, new_returns) = self.load_callable(function);

        match self.context.insert(
            scope,
            function.name.clone(),
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
                } else {
                    function.declared.set(true);
                }
            }
        }

        Ok(())
    }

    fn load_callable<C: ast::Callable>(
        &self,
        signature: &C,
    ) -> (Vec<r#type::Expression>, Vec<r#type::Expression>) {
        let parameters = signature
            .parameters()
            .iter()
            .map(|declaration| &declaration.r#type)
            .map(|r#type| self.load_type(r#type))
            .collect::<Vec<_>>();

        let returns = signature
            .returns()
            .iter()
            .map(|r#type| self.load_type(r#type))
            .collect::<Vec<_>>();

        (parameters, returns)
    }

    fn load_class_field(
        &mut self,
        class: &ast::Identifier,
        declaration: &ast::Declaration,
    ) -> Result<(), error::Error> {
        for (name, r#type) in declaration.iter() {
            let r#type = self.load_type(r#type);
            if let Some((span, _)) = self.context.insert(
                GlobalScope::Class(class.symbol),
                name.clone(),
                Entry::Variable(r#type),
            ) {
                bail!(*name.span, ErrorKind::NameClash(span))
            }
        }

        Ok(())
    }

    fn load_type(&self, r#type: &ast::Type) -> r#type::Expression {
        match r#type {
            ast::Type::Bool(_) => r#type::Expression::Boolean,
            ast::Type::Int(_) => r#type::Expression::Integer,
            ast::Type::Class(variable) => r#type::Expression::Class(self.load_variable(variable)),
            ast::Type::Array(r#type, length, _) => {
                assert!(length.is_none());
                r#type::Expression::Array(Box::new(self.load_type(r#type)))
            }
        }
    }

    // There are two cases to consider here:
    //
    // 1) This generic type is instantiated inside the body of a function, in
    //    which case the monomorphization pass will instantiate the type later.
    //
    // 2) This generic type appears in a signature with no implementation, in
    //    which case the type will be instantiated when we compile the
    //    implementation, and we'll link against it.
    //
    // Unbound type arguments or templates will be caught by `check_type` or
    // the monomorphization pass.
    fn load_variable(&self, variable: &ast::Variable) -> Symbol {
        match &variable.generics {
            None => variable.name.symbol,
            Some(generics) => {
                let generics = generics
                    .iter()
                    .map(|generic| self.load_type(generic))
                    .collect::<Vec<_>>();

                abi::mangle::template(&variable.name.symbol, &generics)
            }
        }
    }
}
