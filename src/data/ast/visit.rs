use crate::data::ast;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Recurse {
    Yes,
    No,
}

macro_rules! visit_mut {
    ($expr:expr) => {
        match $expr {
            Recurse::Yes => (),
            Recurse::No => return,
        }
    };
}

pub trait VisitorMut {
    #[must_use]
    fn visit_interface(&mut self, _interface: &mut ast::Interface) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_item_signature(&mut self, _item: &mut ast::ItemSignature) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_class_signature(&mut self, _class: &mut ast::ClassSignature) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_function_signature(&mut self, _function: &mut ast::FunctionSignature) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_program(&mut self, _program: &mut ast::Program) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_use(&mut self, _use: &mut ast::Use) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_item(&mut self, _item: &mut ast::Item) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_global(&mut self, _global: &mut ast::Global) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_class(&mut self, _class: &mut ast::Class) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_class_template(&mut self, _class: &mut ast::ClassTemplate) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_class_item(&mut self, _item: &mut ast::ClassItem) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_function(&mut self, _function: &mut ast::Function) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_function_template(&mut self, _function: &mut ast::FunctionTemplate) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_statement(&mut self, _statement: &mut ast::Statement) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_declaration(&mut self, _declaration: &mut ast::Declaration) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_multiple_declaration(
        &mut self,
        _declaration: &mut ast::MultipleDeclaration,
    ) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_single_declaration(&mut self, _declaration: &mut ast::SingleDeclaration) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_initialization(&mut self, _initialization: &mut ast::Initialization) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_type(&mut self, _type: &mut ast::Type) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_expression(&mut self, _expression: &mut ast::Expression) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_call(&mut self, _call: &mut ast::Call) -> Recurse {
        Recurse::Yes
    }

    #[must_use]
    fn visit_variable(&mut self, _variable: &mut ast::Variable) -> Recurse {
        Recurse::Yes
    }

    fn visit_identifier(&mut self, _identifier: &mut ast::Identifier) {}
}

impl ast::Interface {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_interface(self));

        let ast::Interface { uses, items } = self;

        uses.iter_mut().for_each(|r#use| r#use.accept_mut(visitor));
        items.iter_mut().for_each(|item| item.accept_mut(visitor));
    }
}

impl ast::ItemSignature {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_item_signature(self));

        match self {
            ast::ItemSignature::Class(class) => class.accept_mut(visitor),
            ast::ItemSignature::ClassTemplate(class) => class.accept_mut(visitor),
            ast::ItemSignature::Function(function) => function.accept_mut(visitor),
            ast::ItemSignature::FunctionTemplate(function) => function.accept_mut(visitor),
        }
    }
}

impl ast::ClassSignature {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_class_signature(self));

        let ast::ClassSignature {
            name,
            extends,
            methods,
            span: _,
        } = self;
        name.accept_mut(visitor);
        extends
            .iter_mut()
            .for_each(|supertype| supertype.accept_mut(visitor));
        methods
            .iter_mut()
            .for_each(|method| method.accept_mut(visitor));
    }
}

impl ast::FunctionSignature {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_function_signature(self));

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
    }
}

impl ast::Program {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_program(self));

        let ast::Program { uses, items } = self;
        uses.iter_mut().for_each(|r#use| r#use.accept_mut(visitor));
        items.iter_mut().for_each(|item| item.accept_mut(visitor));
    }
}

impl ast::Use {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_use(self));

        let ast::Use { name, span: _ } = self;
        visitor.visit_identifier(name);
    }
}

impl ast::Item {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_item(self));

        match self {
            ast::Item::Global(global) => global.accept_mut(visitor),
            ast::Item::Class(class) => class.accept_mut(visitor),
            ast::Item::ClassTemplate(class) => class.accept_mut(visitor),
            ast::Item::Function(function) => function.accept_mut(visitor),
            ast::Item::FunctionTemplate(function) => function.accept_mut(visitor),
        }
    }
}

impl ast::Global {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_global(self));

        match self {
            ast::Global::Declaration(declaration) => declaration.accept_mut(visitor),
            ast::Global::Initialization(initialization) => initialization.accept_mut(visitor),
        }
    }
}

impl ast::Class {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_class(self));

        let ast::Class {
            name,
            extends,
            items,
            span: _,
        } = self;
        name.accept_mut(visitor);
        extends
            .iter_mut()
            .for_each(|supertype| supertype.accept_mut(visitor));
        items.iter_mut().for_each(|item| item.accept_mut(visitor));
    }
}

impl ast::ClassTemplate {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_class_template(self));

        let ast::ClassTemplate {
            name,
            generics,
            items,
            span: _,
        } = self;
        name.accept_mut(visitor);
        generics
            .iter_mut()
            .for_each(|generic| generic.accept_mut(visitor));
        items.iter_mut().for_each(|item| item.accept_mut(visitor));
    }
}

impl ast::ClassItem {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_class_item(self));

        match self {
            ast::ClassItem::Field(field) => field.accept_mut(visitor),
            ast::ClassItem::Method(method) => method.accept_mut(visitor),
        }
    }
}

impl ast::Function {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_function(self));

        let ast::Function {
            name,
            parameters,
            returns,
            statements,
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
    }
}

impl ast::FunctionTemplate {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_function_template(self));

        let ast::FunctionTemplate {
            name,
            generics,
            parameters,
            returns,
            statements,
            span: _,
        } = self;
        name.accept_mut(visitor);
        generics
            .iter_mut()
            .for_each(|generic| generic.accept_mut(visitor));
        parameters
            .iter_mut()
            .for_each(|parameter| parameter.accept_mut(visitor));
        returns
            .iter_mut()
            .for_each(|r#return| r#return.accept_mut(visitor));
        statements.accept_mut(visitor);
    }
}

impl ast::Statement {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_statement(self));

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
    }
}

impl ast::Declaration {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_declaration(self));

        match self {
            ast::Declaration::Multiple(declaration) => declaration.accept_mut(visitor),
            ast::Declaration::Single(declaration) => declaration.accept_mut(visitor),
        }
    }
}

impl ast::MultipleDeclaration {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_multiple_declaration(self));

        let ast::MultipleDeclaration {
            names,
            r#type,
            span: _,
        } = self;
        names.iter_mut().for_each(|name| name.accept_mut(visitor));
        r#type.accept_mut(visitor);
    }
}

impl ast::SingleDeclaration {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_single_declaration(self));

        let ast::SingleDeclaration {
            name,
            r#type,
            span: _,
        } = self;
        name.accept_mut(visitor);
        r#type.accept_mut(visitor);
    }
}

impl ast::Initialization {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_initialization(self));

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
    }
}

impl ast::Type {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_type(self));

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
    }
}

impl ast::Expression {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_expression(self));

        match self {
            ast::Expression::Boolean(_, _)
            | ast::Expression::Character(_, _)
            | ast::Expression::String(_, _)
            | ast::Expression::Integer(_, _)
            | ast::Expression::Null(_)
            | ast::Expression::This(_)
            | ast::Expression::Super(_) => (),
            ast::Expression::Variable(variable) => variable.accept_mut(visitor),
            ast::Expression::Array(expressions, _) => expressions
                .iter_mut()
                .for_each(|expression| expression.accept_mut(visitor)),
            ast::Expression::Binary(_, left, right, _) => {
                left.accept_mut(visitor);
                right.accept_mut(visitor);
            }
            ast::Expression::Unary(_, expression, _) => expression.accept_mut(visitor),
            ast::Expression::Index(array, index, _) => {
                array.accept_mut(visitor);
                index.accept_mut(visitor);
            }
            ast::Expression::Length(array, _) => {
                array.accept_mut(visitor);
            }
            ast::Expression::Call(call) => call.accept_mut(visitor),
            ast::Expression::Dot(_, receiver, identifier, _) => {
                receiver.accept_mut(visitor);
                identifier.accept_mut(visitor);
            }
            ast::Expression::New(variable, _) => variable.accept_mut(visitor),
        }
    }
}

impl ast::Call {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_call(self));

        let ast::Call {
            function,
            arguments,
            span: _,
        } = self;
        function.accept_mut(visitor);
        arguments
            .iter_mut()
            .for_each(|argument| argument.accept_mut(visitor));
    }
}

impl ast::Variable {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visit_mut!(visitor.visit_variable(self));

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
    }
}

impl ast::Identifier {
    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        visitor.visit_identifier(self);
    }
}