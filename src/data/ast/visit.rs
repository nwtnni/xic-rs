use crate::data::ast;

pub trait VisitorMut<T> {
    fn visit_interface(&mut self, _interface: &mut ast::Interface<T>) {}

    fn visit_item_signature(&mut self, _item: &mut ast::ItemSignature<T>) {}

    fn visit_class_signature(&mut self, _class: &mut ast::ClassSignature<T>) {}

    fn visit_function_signature(&mut self, _function: &mut ast::FunctionSignature<T>) {}

    fn visit_program(&mut self, _program: &mut ast::Program<T>) {}

    fn visit_use(&mut self, _use: &mut ast::Use) {}

    fn visit_item(&mut self, _item: &mut ast::Item<T>) {}

    fn visit_global(&mut self, _global: &mut ast::Global<T>) {}

    fn visit_class(&mut self, _class: &mut ast::Class<T>) {}

    fn visit_class_item(&mut self, _item: &mut ast::ClassItem<T>) {}

    fn visit_function(&mut self, _function: &mut ast::Function<T>) {}

    fn visit_statement(&mut self, _statement: &mut ast::Statement<T>) {}

    fn visit_declaration(&mut self, _declaration: &mut ast::Declaration<T>) {}

    fn visit_multiple_declaration(&mut self, _declaration: &mut ast::MultipleDeclaration<T>) {}

    fn visit_single_declaration(&mut self, _declaration: &mut ast::SingleDeclaration<T>) {}

    fn visit_initialization(&mut self, _initialization: &mut ast::Initialization<T>) {}

    fn visit_type(&mut self, _type: &mut ast::Type<T>) {}

    fn visit_expression(&mut self, _expression: &mut ast::Expression<T>) {}

    fn visit_call(&mut self, _call: &mut ast::Call<T>) {}

    fn visit_variable(&mut self, _variable: &mut ast::Variable<T>) {}

    fn visit_identifier(&mut self, _identifier: &mut ast::Identifier) {}
}

impl<T> ast::Interface<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::Interface { uses, items } = self;
        uses.iter_mut().for_each(|r#use| r#use.accept_mut(visitor));
        items.iter_mut().for_each(|item| item.accept_mut(visitor));

        visitor.visit_interface(self);
    }
}

impl<T> ast::ItemSignature<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        match self {
            ast::ItemSignature::Class(class) => class.accept_mut(visitor),
            ast::ItemSignature::ClassTemplate(_) => (),
            ast::ItemSignature::Function(function) => function.accept_mut(visitor),
            ast::ItemSignature::FunctionTemplate(_) => (),
        }

        visitor.visit_item_signature(self);
    }
}

impl<T> ast::ClassSignature<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::ClassSignature {
            r#final: _,
            name,
            extends,
            methods,
            span: _,
        } = self;
        name.accept_mut(visitor);
        if let Some(supertype) = extends {
            supertype.accept_mut(visitor)
        }
        extends
            .iter_mut()
            .for_each(|supertype| supertype.accept_mut(visitor));
        methods
            .iter_mut()
            .for_each(|method| method.accept_mut(visitor));

        visitor.visit_class_signature(self);
    }
}

impl<T> ast::FunctionSignature<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::FunctionSignature {
            name,
            parameters,
            returns,
            span: _,
        } = self;
        name.accept_mut(visitor);
        parameters
            .iter_mut()
            .for_each(|parameter| parameter.accept_mut(visitor));
        returns
            .iter_mut()
            .for_each(|r#return| r#return.accept_mut(visitor));

        visitor.visit_function_signature(self);
    }
}

impl<T> ast::Program<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::Program { uses, items } = self;
        uses.iter_mut().for_each(|r#use| r#use.accept_mut(visitor));
        items.iter_mut().for_each(|item| item.accept_mut(visitor));

        visitor.visit_program(self);
    }
}

impl ast::Use {
    pub fn accept_mut<V: VisitorMut<T>, T>(&mut self, visitor: &mut V) {
        let ast::Use { name, span: _ } = self;
        visitor.visit_identifier(name);

        visitor.visit_use(self);
    }
}

impl<T> ast::Item<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        match self {
            ast::Item::Global(global) => global.accept_mut(visitor),
            ast::Item::Class(class) => class.accept_mut(visitor),
            ast::Item::ClassTemplate(_) => (),
            ast::Item::Function(function) => function.accept_mut(visitor),
            ast::Item::FunctionTemplate(_) => (),
        }

        visitor.visit_item(self);
    }
}

impl<T> ast::Global<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        match self {
            ast::Global::Declaration(declaration) => declaration.accept_mut(visitor),
            ast::Global::Initialization(initialization) => initialization.accept_mut(visitor),
        }

        visitor.visit_global(self);
    }
}

impl<T> ast::Class<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::Class {
            r#final: _,
            name,
            extends,
            items,
            provenance: _,
            declared: _,
            span: _,
        } = self;
        name.accept_mut(visitor);
        if let Some(supertype) = extends {
            supertype.accept_mut(visitor)
        }
        items.iter_mut().for_each(|item| item.accept_mut(visitor));

        visitor.visit_class(self);
    }
}

impl<T> ast::ClassItem<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        match self {
            ast::ClassItem::Field(field) => field.accept_mut(visitor),
            ast::ClassItem::Method(method) => method.accept_mut(visitor),
        }

        visitor.visit_class_item(self);
    }
}

impl<T> ast::Function<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::Function {
            name,
            parameters,
            returns,
            statements,
            provenance: _,
            declared: _,
            span: _,
        } = self;
        name.accept_mut(visitor);
        parameters
            .iter_mut()
            .for_each(|parameter| parameter.accept_mut(visitor));
        returns
            .iter_mut()
            .for_each(|r#return| r#return.accept_mut(visitor));
        statements.accept_mut(visitor);

        visitor.visit_function(self);
    }
}

impl<T> ast::Statement<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        match self {
            ast::Statement::Assignment(destination, source, _) => {
                destination.accept_mut(visitor);
                source.accept_mut(visitor);
            }
            ast::Statement::Call(call) => call.accept_mut(visitor),
            ast::Statement::Initialization(initialization) => initialization.accept_mut(visitor),
            ast::Statement::Declaration(declaration, _) => declaration.accept_mut(visitor),
            ast::Statement::Return(r#returns, _) => r#returns
                .iter_mut()
                .for_each(|r#return| r#return.accept_mut(visitor)),
            ast::Statement::Sequence(statements, _) => statements
                .iter_mut()
                .for_each(|statement| statement.accept_mut(visitor)),
            ast::Statement::If(condition, r#if, r#else, _) => {
                condition.accept_mut(visitor);
                r#if.accept_mut(visitor);
                if let Some(r#else) = r#else {
                    r#else.accept_mut(visitor);
                }
            }
            ast::Statement::While(_, condition, r#while, _) => {
                condition.accept_mut(visitor);
                r#while.accept_mut(visitor);
            }
            ast::Statement::Break(_) => (),
        }

        visitor.visit_statement(self);
    }
}

impl<T> ast::Declaration<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        match self {
            ast::Declaration::Multiple(declaration) => declaration.accept_mut(visitor),
            ast::Declaration::Single(declaration) => declaration.accept_mut(visitor),
        }

        visitor.visit_declaration(self);
    }
}

impl<T> ast::MultipleDeclaration<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::MultipleDeclaration {
            names,
            r#type,
            span: _,
        } = self;
        names.iter_mut().for_each(|name| name.accept_mut(visitor));
        r#type.accept_mut(visitor);

        visitor.visit_multiple_declaration(self);
    }
}

impl<T> ast::SingleDeclaration<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::SingleDeclaration {
            name,
            r#type,
            span: _,
        } = self;
        name.accept_mut(visitor);
        r#type.accept_mut(visitor);

        visitor.visit_single_declaration(self);
    }
}

impl<T> ast::Initialization<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::Initialization {
            declarations,
            expression,
            span: _,
        } = self;
        declarations
            .iter_mut()
            .flatten()
            .for_each(|declaration| declaration.accept_mut(visitor));
        expression.accept_mut(visitor);

        visitor.visit_initialization(self);
    }
}

impl<T> ast::Type<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        match self {
            ast::Type::Bool(_) => (),
            ast::Type::Int(_) => (),
            ast::Type::Class(variable) => variable.accept_mut(visitor),
            ast::Type::Array(r#type, length, _) => {
                r#type.accept_mut(visitor);
                if let Some(length) = length {
                    length.accept_mut(visitor);
                }
            }
        }

        visitor.visit_type(self);
    }
}

impl<T> ast::Expression<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        match self {
            ast::Expression::Boolean(_, _)
            | ast::Expression::Character(_, _)
            | ast::Expression::String(_, _)
            | ast::Expression::Integer(_, _)
            | ast::Expression::Null(_)
            | ast::Expression::This(_, _)
            | ast::Expression::Super(_, _) => (),
            ast::Expression::Variable(variable, _) => variable.accept_mut(visitor),
            ast::Expression::Array(expressions, _, _) => expressions
                .iter_mut()
                .for_each(|expression| expression.accept_mut(visitor)),
            ast::Expression::Binary(_, left, right, _, _) => {
                left.accept_mut(visitor);
                right.accept_mut(visitor);
            }
            ast::Expression::Unary(_, expression, _, _) => expression.accept_mut(visitor),
            ast::Expression::Index(array, index, _, _) => {
                array.accept_mut(visitor);
                index.accept_mut(visitor);
            }
            ast::Expression::Length(array, _) => {
                array.accept_mut(visitor);
            }
            ast::Expression::Call(call) => call.accept_mut(visitor),
            ast::Expression::Dot(receiver, identifier, _, _) => {
                receiver.accept_mut(visitor);
                identifier.accept_mut(visitor);
            }
            ast::Expression::New(variable, _) => variable.accept_mut(visitor),
        }

        visitor.visit_expression(self);
    }
}

impl<T> ast::Call<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::Call {
            function,
            arguments,
            span: _,
        } = self;
        function.accept_mut(visitor);
        arguments
            .iter_mut()
            .for_each(|argument| argument.accept_mut(visitor));

        visitor.visit_call(self);
    }
}

impl<T> ast::Variable<T> {
    pub fn accept_mut<V: VisitorMut<T>>(&mut self, visitor: &mut V) {
        let ast::Variable {
            name,
            generics,
            span: _,
        } = self;
        name.accept_mut(visitor);
        generics
            .iter_mut()
            .flatten()
            .for_each(|r#type| r#type.accept_mut(visitor));

        visitor.visit_variable(self);
    }
}

impl ast::Identifier {
    pub fn accept_mut<V: VisitorMut<T>, T>(&mut self, visitor: &mut V) {
        visitor.visit_identifier(self);
    }
}
