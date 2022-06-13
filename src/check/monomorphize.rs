use crate::abi;
use crate::check::check::Checker;
use crate::check::GlobalScope;
use crate::data::ast;
use crate::Map;

#[allow(unused)]
impl Checker {
    pub(super) fn monomorphize_program(&mut self, program: &mut ast::Program) {
        let mut monomorphizer = Monomorphizer {
            functions: Map::default(),
            classes: Map::default(),
            stack: Vec::new(),
            checker: self,
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
    }
}

struct Monomorphizer<'a> {
    functions: Map<ast::Identifier, Map<Vec<ast::Type>, Option<ast::Function>>>,
    classes: Map<ast::Identifier, Map<Vec<ast::Type>, Option<ast::Class>>>,
    stack: Vec<Map<ast::Identifier, ast::Type>>,
    checker: &'a mut Checker,
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
                .cloned()
            {
                match (&variable.generics, substitute) {
                    (None, substitute) => *r#type = substitute,
                    // Forward type arguments to any functor arguments
                    (
                        Some(_),
                        ast::Type::Class(ast::Variable {
                            name,
                            generics: None,
                            span: _,
                        }),
                    ) => {
                        variable.name = name;
                    }
                    (Some(_), _) => todo!("Generic arguments to non-functor"),
                }
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

        let template = self
            .checker
            .context
            .get_class_template(name)
            .cloned()
            .unwrap();

        self.classes
            .entry(template.name.clone())
            .or_default()
            .insert(generics.to_vec(), None);

        let mut instantiation = ast::Class {
            name: ast::Identifier {
                symbol: abi::mangle::template(&template.name.symbol, generics),
                span: template.name.span.clone(),
            },
            extends: None,
            items: template.items,
            span: template.span,
        };

        // TODO: check that (1) type parameters are unique and (2) type argument counts match
        self.stack.push(
            template
                .generics
                .clone()
                .into_iter()
                .zip(generics.iter().cloned())
                .collect(),
        );
        instantiation.accept_mut(self);
        self.stack.pop();

        self.checker.load_class(&instantiation).unwrap();
        self.classes[&template.name][&*generics] = Some(instantiation);
    }

    fn instantiate_function_template(&mut self, name: &ast::Identifier, generics: &[ast::Type]) {
        // Already instantiated, so just rewrite
        if self.functions.get(name).map_or(false, |instantiations| {
            instantiations.contains_key(generics)
        }) {
            return;
        }

        let template = self
            .checker
            .context
            .get_function_template(name)
            .cloned()
            .unwrap();

        self.functions
            .entry(template.name.clone())
            .or_default()
            .insert(generics.to_vec(), None);

        let mut instantiation = ast::Function {
            name: ast::Identifier {
                symbol: abi::mangle::template(&template.name.symbol, generics),
                span: template.name.span.clone(),
            },
            parameters: template.parameters,
            returns: template.returns,
            statements: template.statements,
            span: template.span,
        };

        // TODO: check that (1) type parameters are unique and (2) type argument counts match
        self.stack.push(
            template
                .generics
                .clone()
                .into_iter()
                .zip(generics.iter().cloned())
                .collect(),
        );
        instantiation.accept_mut(self);
        self.stack.pop();

        self.checker
            .load_function(GlobalScope::Global, &instantiation)
            .unwrap();
        self.functions[&template.name][&*generics] = Some(instantiation);
    }
}
