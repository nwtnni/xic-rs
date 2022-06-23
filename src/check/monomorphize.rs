use std::cell::Cell;

use crate::abi;
use crate::check::check::Checker;
use crate::check::Error;
use crate::check::ErrorKind;
use crate::check::GlobalScope;
use crate::data::ast;
use crate::data::span::Span;
use crate::Map;

impl Checker {
    pub(super) fn monomorphize_program(
        &mut self,
        program: &mut ast::Program<()>,
    ) -> Result<(), Error> {
        let mut monomorphizer = Monomorphizer {
            functions: Map::default(),
            classes: Map::default(),
            arguments: Vec::new(),
            checker: self,
            error: None,
        };

        program.accept_mut(&mut monomorphizer);

        program.items.retain(|item| {
            !matches!(
                item,
                ast::Item::ClassTemplate(_) | ast::Item::FunctionTemplate(_)
            )
        });

        program.items.extend(
            monomorphizer
                .classes
                .into_values()
                .flat_map(Map::into_values)
                .flatten()
                .map(ast::Item::Class),
        );

        program.items.extend(
            monomorphizer
                .functions
                .into_values()
                .flat_map(Map::into_values)
                .flatten()
                .map(ast::Item::Function),
        );

        match monomorphizer.error {
            None => Ok(()),
            Some(error) => Err(error),
        }
    }
}

struct Monomorphizer<'a> {
    functions: Map<ast::Identifier, Map<Vec<ast::Type<()>>, Option<ast::Function<()>>>>,
    classes: Map<ast::Identifier, Map<Vec<ast::Type<()>>, Option<ast::Class<()>>>>,
    arguments: Vec<(Span, Map<ast::Identifier, ast::Type<()>>)>,
    checker: &'a mut Checker,
    error: Option<Error>,
}

impl<'a> ast::VisitorMut<()> for Monomorphizer<'a> {
    fn visit_class(&mut self, class: &mut ast::Class<()>) {
        if self.error.is_some() {
            return;
        }

        if let Some(supertype) = class.extends.as_mut() {
            self.monomorphize_class(supertype);
        }
    }

    fn visit_call(&mut self, call: &mut ast::Call<()>) {
        if self.error.is_some() {
            return;
        }

        if let ast::Expression::Variable(variable, ()) = &mut *call.function {
            self.monomorphize(Self::instantiate_function_template, variable);
        }
    }

    fn visit_type(&mut self, r#type: &mut ast::Type<()>) {
        if self.error.is_some() {
            return;
        }

        if let ast::Type::Class(variable) = r#type {
            if let Some(substitute) = self
                .arguments
                .last()
                .and_then(|(_, arguments)| arguments.get(&variable.name))
                .cloned()
            {
                *r#type = substitute;
            }
        }

        if let ast::Type::Class(variable) = r#type {
            self.monomorphize_class(variable);
        }
    }

    fn visit_expression(&mut self, expression: &mut ast::Expression<()>) {
        if self.error.is_some() {
            return;
        }

        if let ast::Expression::New(variable, _) = expression {
            self.monomorphize_class(variable);
        }
    }
}

impl<'a> Monomorphizer<'a> {
    fn monomorphize_class(&mut self, variable: &mut ast::Variable<()>) {
        self.monomorphize(Self::instantiate_class_template, variable);
    }

    fn monomorphize(
        &mut self,
        instantiate: fn(&mut Self, &ast::Identifier, &[ast::Type<()>], &Span),
        variable: &mut ast::Variable<()>,
    ) {
        let generics = match &mut variable.generics {
            Some(generics) => generics,
            None => return,
        };

        for generic in &*generics {
            if let Err(error) = self.checker.check_type(generic.clone()) {
                self.error = Some(error);
                return;
            }
        }

        instantiate(self, &variable.name, generics, &variable.span);

        variable.name.symbol = abi::mangle::template(&variable.name.symbol, generics);
        variable.generics = None;
    }

    fn instantiate_class_template(
        &mut self,
        name: &ast::Identifier,
        generics: &[ast::Type<()>],
        span: &Span,
    ) {
        // Already instantiated, so just rewrite
        if self.classes.get(name).map_or(false, |instantiations| {
            instantiations.contains_key(generics)
        }) {
            return;
        }

        let template = match self.checker.context.get_class_template(name) {
            Some(template) if template.generics.len() == generics.len() => template.clone(),
            Some(template) => {
                self.error = Some(Error::new(
                    *span,
                    ErrorKind::TemplateArgumentMismatch {
                        span: *template.name.span,
                        expected: template.generics.len(),
                        found: generics.len(),
                    },
                ));
                return;
            }
            None => {
                self.error = Some(Error::new(
                    *name.span,
                    ErrorKind::UnboundClassTemplate(name.symbol),
                ));
                return;
            }
        };

        self.classes
            .entry(template.name.clone())
            .or_default()
            .insert(generics.to_vec(), None);

        self.arguments.push((
            *span,
            template
                .generics
                .clone()
                .into_iter()
                .zip(generics.iter().cloned())
                .collect(),
        ));

        let mut instantiation = ast::Class {
            r#final: template.r#final,
            name: ast::Identifier {
                symbol: abi::mangle::template(&template.name.symbol, generics),
                span: template.name.span.clone(),
            },
            extends: template.extends,
            items: template.items,
            provenance: self
                .arguments
                .iter()
                .map(|(span, _)| span)
                .copied()
                .collect(),
            declared: Cell::new(false),
            span: template.span,
        };

        instantiation.accept_mut(self);
        self.arguments.pop();

        self.checker.load_class(&instantiation).unwrap();
        self.classes[&template.name][&*generics] = Some(instantiation);
    }

    fn instantiate_function_template(
        &mut self,
        name: &ast::Identifier,
        generics: &[ast::Type<()>],
        span: &Span,
    ) {
        // Already instantiated, so just rewrite
        if self.functions.get(name).map_or(false, |instantiations| {
            instantiations.contains_key(generics)
        }) {
            return;
        }

        let template = match self.checker.context.get_function_template(name) {
            Some(template) if template.generics.len() == generics.len() => template.clone(),
            Some(template) => {
                self.error = Some(Error::new(
                    *span,
                    ErrorKind::TemplateArgumentMismatch {
                        span: *template.name.span,
                        expected: template.generics.len(),
                        found: generics.len(),
                    },
                ));
                return;
            }
            None => {
                self.error = Some(Error::new(
                    *name.span,
                    ErrorKind::UnboundFunTemplate(name.symbol),
                ));
                return;
            }
        };

        self.functions
            .entry(template.name.clone())
            .or_default()
            .insert(generics.to_vec(), None);

        self.arguments.push((
            *span,
            template
                .generics
                .clone()
                .into_iter()
                .zip(generics.iter().cloned())
                .collect(),
        ));

        let mut instantiation = ast::Function {
            name: ast::Identifier {
                symbol: abi::mangle::template(&template.name.symbol, generics),
                span: template.name.span.clone(),
            },
            parameters: template.parameters,
            returns: template.returns,
            statements: template.statements,
            provenance: self
                .arguments
                .iter()
                .map(|(span, _)| span)
                .copied()
                .collect(),
            declared: Cell::new(false),
            span: template.span,
        };

        instantiation.accept_mut(self);
        self.arguments.pop();

        self.checker
            .load_function(GlobalScope::Global, &instantiation)
            .unwrap();
        self.functions[&template.name][&*generics] = Some(instantiation);
    }
}
