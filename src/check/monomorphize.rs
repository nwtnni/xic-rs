use crate::abi;
use crate::check::check::Checker;
use crate::data::ast;
use crate::Map;

#[allow(unused)]
pub(super) fn monomorphize_program(checker: &Checker, program: &mut ast::Program) {
    let mut monomorphizer = Monomorphizer {
        functions: Map::default(),
        classes: Map::default(),
        stack: Vec::new(),
        checker,
    };

    program.accept_mut(&mut monomorphizer);

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
}

struct Monomorphizer<'a> {
    functions: Map<ast::Identifier, Map<Vec<ast::Type>, Option<ast::Function>>>,
    classes: Map<ast::Identifier, Map<Vec<ast::Type>, Option<ast::Class>>>,
    stack: Vec<Map<ast::Identifier, ast::Type>>,
    checker: &'a Checker,
}

impl<'a> ast::VisitorMut for Monomorphizer<'a> {
    fn visit_call(&mut self, call: &mut ast::Call) {
        if let ast::Expression::Variable(variable) = &mut *call.function {
            self.monomorphize(Self::instantiate_function_template, variable);
        }
    }

    fn visit_type(&mut self, r#type: &mut ast::Type) {
        if let ast::Type::Class(variable) = r#type {
            if let Some(substitute) = self
                .stack
                .last()
                .and_then(|arguments| arguments.get(&variable.name))
            {
                *r#type = substitute.clone();
                // FIXME: is this necessary?
                r#type.accept_mut(self);
            }
        }

        if let ast::Type::Class(variable) = r#type {
            self.monomorphize(Self::instantiate_class_template, variable);
        }
    }

    fn visit_expression(&mut self, expression: &mut ast::Expression) {
        if let ast::Expression::New(variable, _) = expression {
            self.monomorphize(Self::instantiate_class_template, variable);
        }
    }
}

impl<'a> Monomorphizer<'a> {
    fn monomorphize(
        &mut self,
        instantiate: fn(&mut Self, &ast::Identifier, &[ast::Type]),
        variable: &mut ast::Variable,
    ) {
        let generics = match &mut variable.generics {
            Some(generics) => generics,
            None => return,
        };

        instantiate(self, &variable.name, generics);

        variable.name.symbol = abi::mangle::template(&variable.name.symbol, generics);
        variable.generics = None;
    }

    fn instantiate_class_template(&mut self, name: &ast::Identifier, generics: &[ast::Type]) {
        // Already instantiated, so just rewrite
        if self.classes.get(name).map_or(false, |instantiations| {
            instantiations.contains_key(generics)
        }) {
            return;
        }

        let mut template = self
            .checker
            .context
            .get_class_template(name)
            .cloned()
            .unwrap();

        self.classes
            .entry(template.name.clone())
            .or_default()
            .insert(generics.to_vec(), None);

        // TODO: check that (1) type parameters are unique and (2) type argument counts match
        self.stack.push(
            template
                .generics
                .clone()
                .into_iter()
                .zip(generics.iter().cloned())
                .collect(),
        );

        // Instantiate template
        template.accept_mut(self);

        self.classes[&template.name][&*generics] = Some(ast::Class {
            name: ast::Identifier {
                symbol: abi::mangle::template(&template.name.symbol, generics),
                span: template.name.span.clone(),
            },
            extends: None,
            items: template.items,
            span: template.span,
        });
    }

    fn instantiate_function_template(&mut self, name: &ast::Identifier, generics: &[ast::Type]) {
        // Already instantiated, so just rewrite
        if self.functions.get(name).map_or(false, |instantiations| {
            instantiations.contains_key(generics)
        }) {
            return;
        }

        let mut template = self
            .checker
            .context
            .get_function_template(name)
            .cloned()
            .unwrap();

        self.functions
            .entry(template.name.clone())
            .or_default()
            .insert(generics.to_vec(), None);
        self.stack.push(
            template
                .generics
                .clone()
                .into_iter()
                .zip(generics.iter().cloned())
                .collect(),
        );

        // Instantiate template
        template.accept_mut(self);

        self.functions[&template.name][&*generics] = Some(ast::Function {
            name: ast::Identifier {
                symbol: abi::mangle::template(&template.name.symbol, generics),
                span: template.name.span.clone(),
            },
            parameters: template.parameters,
            returns: template.returns,
            statements: template.statements,
            span: template.span,
        });
    }
}
