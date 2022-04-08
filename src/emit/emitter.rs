use std::collections::HashMap;

use crate::check;
use crate::constants;
use crate::data::ast;
use crate::data::hir;
use crate::data::ir;
use crate::data::operand;
use crate::data::r#type;
use crate::util::symbol;

#[derive(Debug)]
pub struct Emitter<'env> {
    context: &'env check::Context,
    data: HashMap<symbol::Symbol, operand::Label>,
    functions: HashMap<symbol::Symbol, symbol::Symbol>,
}

impl<'env> Emitter<'env> {
    pub fn new(context: &'env check::Context) -> Self {
        Emitter {
            context,
            data: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn emit_unit(
        mut self,
        path: &std::path::Path,
        ast: &ast::Program,
    ) -> ir::Unit<hir::Function> {
        let mut functions = HashMap::with_capacity(ast.functions.len());

        for fun in &ast.functions {
            let name = self.mangle_function(fun.name);
            let hir = self.emit_function(fun);
            functions.insert(name, hir);
        }

        ir::Unit {
            name: symbol::intern(path.to_string_lossy().trim_start_matches("./")),
            functions,
            data: self.data,
        }
    }

    fn emit_function(&mut self, function: &ast::Function) -> hir::Function {
        let mut variables = HashMap::default();
        let mut statements = Vec::new();

        for (index, parameter) in function.parameters.iter().enumerate() {
            let argument = hir::Expression::Temporary(operand::Temporary::Argument(index));
            let declaration = self.emit_declaration(parameter, &mut variables);
            statements.push(hir::Statement::Move(declaration, argument));
        }

        let name = self.mangle_function(function.name);
        let statement = self.emit_statement(&function.statements, &mut variables);

        statements.push(statement);

        hir::Function {
            name,
            statements: hir::Statement::Sequence(statements),
        }
    }

    fn emit_expression(
        &mut self,
        expression: &ast::Expression,
        variables: &HashMap<symbol::Symbol, operand::Temporary>,
    ) -> hir::Tree {
        use ast::Expression::*;
        match expression {
            Boolean(false, _) => hir::Expression::Integer(0).into(),
            Boolean(true, _) => hir::Expression::Integer(1).into(),
            Integer(integer, _) => hir::Expression::Integer(*integer).into(),
            Character(character, _) => hir::Expression::Integer(*character as i64).into(),
            String(string, _) => {
                let symbol = symbol::intern(string);
                let label = *self
                    .data
                    .entry(symbol)
                    .or_insert_with(|| operand::Label::fresh("string"));

                hir::Expression::Label(label).into()
            }
            Variable(variable, _) => hir::Expression::Temporary(variables[variable]).into(),
            Array(expressions, _) => {
                let alloc = Self::emit_alloc(expressions.len());
                let base = hir::Expression::Temporary(operand::Temporary::fresh("array"));

                let mut statements = vec![
                    alloc,
                    hir::Statement::Move(
                        base.clone(),
                        hir::Expression::Temporary(operand::Temporary::Return(0)),
                    ),
                    hir::Statement::Move(
                        hir::Expression::Memory(Box::new(base.clone())),
                        hir::Expression::Integer(expressions.len() as i64),
                    ),
                ];

                for (index, expression) in expressions.iter().enumerate() {
                    let expression = self.emit_expression(expression, variables).into();
                    let offset =
                        hir::Expression::Integer((index + 1) as i64 * constants::WORD_SIZE);
                    let address = hir::Expression::Memory(Box::new(hir::Expression::Binary(
                        ir::Binary::Add,
                        Box::new(base.clone()),
                        Box::new(offset),
                    )));
                    statements.push(hir::Statement::Move(address, expression));
                }

                hir::Expression::Sequence(
                    Box::new(hir::Statement::Sequence(statements)),
                    Box::new(hir::Expression::Binary(
                        ir::Binary::Add,
                        Box::new(base),
                        Box::new(hir::Expression::Integer(constants::WORD_SIZE)),
                    )),
                )
                .into()
            }

            Binary(binary, left, right, _) => {
                let binary = ir::Binary::from(*binary);
                let left = self.emit_expression(left, variables);
                let right = self.emit_expression(right, variables);

                match binary {
                    ir::Binary::Mul
                    | ir::Binary::Hul
                    | ir::Binary::Mod
                    | ir::Binary::Div
                    | ir::Binary::Add
                    | ir::Binary::Sub => hir::Expression::Binary(
                        binary,
                        Box::new(left.into()),
                        Box::new(right.into()),
                    )
                    .into(),
                    ir::Binary::Lt
                    | ir::Binary::Le
                    | ir::Binary::Ge
                    | ir::Binary::Gt
                    | ir::Binary::Ne
                    | ir::Binary::Eq => hir::Tree::Condition(Box::new(move |r#true, r#false| {
                        hir::Statement::CJump(
                            hir::Expression::Binary(
                                binary,
                                Box::new(left.into()),
                                Box::new(right.into()),
                            ),
                            r#true,
                            r#false,
                        )
                    })),
                    ir::Binary::And => hir::Tree::Condition(Box::new(move |r#true, r#false| {
                        let and = operand::Label::fresh("and");
                        hir::Statement::Sequence(vec![
                            hir::Condition::from(left)(and, r#false),
                            hir::Statement::Label(and),
                            hir::Condition::from(right)(r#true, r#false),
                        ])
                    })),
                    ir::Binary::Or => hir::Tree::Condition(Box::new(move |r#true, r#false| {
                        let or = operand::Label::fresh("or");
                        hir::Statement::Sequence(vec![
                            hir::Condition::from(left)(r#true, or),
                            hir::Statement::Label(or),
                            hir::Condition::from(right)(r#true, r#false),
                        ])
                    })),
                    ir::Binary::Xor | ir::Binary::Ls | ir::Binary::Rs | ir::Binary::ARs => {
                        unreachable!("[INTERNAL ERROR]: no XOR, LSHIFT, RSHIFT in AST")
                    }
                }
            }

            Unary(ast::Unary::Neg, expression, _) => hir::Expression::Binary(
                ir::Binary::Sub,
                Box::new(hir::Expression::Integer(0)),
                Box::new(self.emit_expression(expression, variables).into()),
            )
            .into(),
            Unary(ast::Unary::Not, expression, _) => hir::Expression::Binary(
                ir::Binary::Xor,
                Box::new(hir::Expression::Integer(1)),
                Box::new(self.emit_expression(expression, variables).into()),
            )
            .into(),

            Index(array, array_index, _) => {
                let base = operand::Temporary::fresh("base");
                let index = operand::Temporary::fresh("index");

                let length = hir::Expression::Memory(Box::new(hir::Expression::Binary(
                    ir::Binary::Sub,
                    Box::new(hir::Expression::Temporary(base)),
                    Box::new(hir::Expression::Integer(constants::WORD_SIZE)),
                )));

                let address = hir::Expression::Binary(
                    ir::Binary::Add,
                    Box::new(hir::Expression::Temporary(base)),
                    Box::new(hir::Expression::Binary(
                        ir::Binary::Mul,
                        Box::new(hir::Expression::Temporary(index)),
                        Box::new(hir::Expression::Integer(constants::WORD_SIZE)),
                    )),
                );

                let low = operand::Label::fresh("low");
                let high = operand::Label::fresh("high");
                let out = operand::Label::fresh("out");
                let r#in = operand::Label::fresh("in");

                let bounds = hir::Statement::Sequence(vec![
                    hir::Statement::Move(
                        hir::Expression::Temporary(base),
                        self.emit_expression(&*array, variables).into(),
                    ),
                    hir::Statement::Move(
                        hir::Expression::Temporary(index),
                        self.emit_expression(&*array_index, variables).into(),
                    ),
                    hir::Statement::CJump(
                        hir::Expression::Binary(
                            ir::Binary::Lt,
                            Box::new(hir::Expression::Temporary(index)),
                            Box::new(hir::Expression::Integer(0)),
                        ),
                        out,
                        low,
                    ),
                    hir::Statement::Label(low),
                    hir::Statement::CJump(
                        hir::Expression::Binary(
                            ir::Binary::Ge,
                            Box::new(hir::Expression::Temporary(index)),
                            Box::new(length),
                        ),
                        out,
                        high,
                    ),
                    hir::Statement::Label(high),
                    hir::Statement::Jump(hir::Expression::Label(r#in)),
                    hir::Statement::Label(out),
                    hir::Statement::Call(hir::Call {
                        name: Box::new(hir::Expression::Label(operand::Label::Fixed(
                            symbol::intern(constants::XI_OUT_OF_BOUNDS),
                        ))),
                        arguments: Vec::new(),
                    }),
                    hir::Statement::Label(r#in),
                ]);

                hir::Expression::Sequence(Box::new(bounds), Box::new(address)).into()
            }
            Call(call) if call.name == symbol::intern("length") => {
                let address = self.emit_expression(&call.arguments[0], variables).into();
                hir::Expression::Memory(Box::new(hir::Expression::Binary(
                    ir::Binary::Sub,
                    Box::new(address),
                    Box::new(hir::Expression::Integer(constants::WORD_SIZE)),
                )))
                .into()
            }
            Call(call) => hir::Expression::Call(self.emit_call(call, variables)).into(),
        }
    }

    fn emit_call(
        &mut self,
        call: &ast::Call,
        variables: &HashMap<symbol::Symbol, operand::Temporary>,
    ) -> hir::Call {
        hir::Call {
            name: Box::new(hir::Expression::Label(operand::Label::Fixed(
                self.mangle_function(call.name),
            ))),
            arguments: call
                .arguments
                .iter()
                .map(|argument| self.emit_expression(argument, variables).into())
                .collect(),
        }
    }

    fn emit_alloc(length: usize) -> hir::Statement {
        let label = operand::Label::Fixed(symbol::intern(constants::XI_ALLOC));
        let alloc = hir::Expression::Label(label);
        let bytes = hir::Expression::Integer((length + 1) as i64 * constants::WORD_SIZE);
        hir::Statement::Call(hir::Call {
            name: Box::new(alloc),
            arguments: vec![bytes],
        })
    }

    fn emit_declaration(
        &mut self,
        declaration: &ast::Declaration,
        variables: &mut HashMap<symbol::Symbol, operand::Temporary>,
    ) -> hir::Expression {
        let fresh = operand::Temporary::fresh("t");
        variables.insert(declaration.name, fresh);
        hir::Expression::Temporary(fresh)
    }

    fn emit_statement(
        &mut self,
        statement: &ast::Statement,
        variables: &mut HashMap<symbol::Symbol, operand::Temporary>,
    ) -> hir::Statement {
        use ast::Statement::*;
        match statement {
            Assignment(left, right, _) => {
                let lhs = self.emit_expression(left, variables).into();
                let rhs = self.emit_expression(right, variables).into();
                hir::Statement::Move(lhs, rhs)
            }
            Call(call) => hir::Statement::Call(self.emit_call(call, variables)),
            Initialization(declarations, ast::Expression::Call(call), _) => {
                let mut statements = vec![hir::Statement::Call(self.emit_call(call, variables))];

                for (index, declaration) in declarations.iter().enumerate() {
                    if let Some(declaration) = declaration {
                        let variable = self.emit_declaration(declaration, variables);
                        let r#return =
                            hir::Expression::Temporary(operand::Temporary::Return(index));
                        statements.push(hir::Statement::Move(variable, r#return));
                    }
                }

                hir::Statement::Sequence(statements)
            }
            Initialization(declarations, expression, _) => {
                assert!(declarations.len() == 1 && declarations[0].is_some());
                let declaration = declarations[0].as_ref().unwrap();
                let variable = self.emit_declaration(declaration, variables);
                let expression = self.emit_expression(expression, variables).into();
                hir::Statement::Move(variable, expression)
            }
            Declaration(declaration, _) => hir::Statement::Move(
                self.emit_declaration(declaration, variables),
                hir::Expression::Integer(0),
            ),
            Return(expressions, _) => hir::Statement::Return(
                expressions
                    .iter()
                    .map(|expression| self.emit_expression(expression, variables).into())
                    .collect(),
            ),
            Sequence(statements, _) => hir::Statement::Sequence(
                statements
                    .iter()
                    .map(|statement| self.emit_statement(statement, variables))
                    .collect(),
            ),
            If(condition, r#if, None, _) => {
                let r#true = operand::Label::fresh("true");
                let r#false = operand::Label::fresh("false");
                hir::Statement::Sequence(vec![
                    hir::Condition::from(self.emit_expression(condition, variables))(
                        r#true, r#false,
                    ),
                    hir::Statement::Label(r#true),
                    self.emit_statement(r#if, variables),
                    hir::Statement::Label(r#false),
                ])
            }
            If(condition, r#if, Some(r#else), _) => {
                let r#true = operand::Label::fresh("true");
                let r#false = operand::Label::fresh("false");
                let end = operand::Label::fresh("endif");
                hir::Statement::Sequence(vec![
                    hir::Condition::from(self.emit_expression(condition, variables))(
                        r#true, r#false,
                    ),
                    hir::Statement::Label(r#true),
                    self.emit_statement(r#if, variables),
                    hir::Statement::Jump(hir::Expression::Label(end)),
                    hir::Statement::Label(r#false),
                    self.emit_statement(r#else, variables),
                    hir::Statement::Label(end),
                ])
            }
            While(condition, statements, _) => {
                let r#while = operand::Label::fresh("while");
                let r#true = operand::Label::fresh("true");
                let r#false = operand::Label::fresh("false");
                hir::Statement::Sequence(vec![
                    hir::Statement::Label(r#while),
                    hir::Condition::from(self.emit_expression(condition, variables))(
                        r#true, r#false,
                    ),
                    self.emit_statement(statements, variables),
                    hir::Statement::Jump(hir::Expression::Label(r#while)),
                    hir::Statement::Label(r#false),
                ])
            }
        }
    }

    fn mangle_function(&mut self, name: symbol::Symbol) -> symbol::Symbol {
        if let Some(mangled) = self.functions.get(&name) {
            return *mangled;
        }

        let (parameters, returns) = match self.context.get(name) {
            Some(check::Entry::Function(parameters, returns))
            | Some(check::Entry::Signature(parameters, returns)) => (parameters, returns),
            _ => panic!("[INTERNAL ERROR]: type checking failed"),
        };

        let mut mangled = format!("_I{}_", symbol::resolve(name).replace('_', "__"),);

        match returns.as_slice() {
            [] => mangled.push('p'),
            [r#type] => {
                Self::mangle_type(r#type, &mut mangled);
            }
            types => {
                mangled.push('t');
                mangled.push_str(&types.len().to_string());
                for r#type in types {
                    Self::mangle_type(r#type, &mut mangled);
                }
            }
        }

        for parameter in parameters {
            Self::mangle_type(parameter, &mut mangled);
        }

        let mangled = symbol::intern(mangled);
        self.functions.insert(name, mangled);
        mangled
    }

    fn mangle_type(r#type: &r#type::Expression, mangled: &mut String) {
        match r#type {
            r#type::Expression::Any => panic!("[INTERNAL ERROR]: any type in IR"),
            r#type::Expression::Integer => mangled.push('i'),
            r#type::Expression::Boolean => mangled.push('b'),
            r#type::Expression::Array(r#type) => {
                mangled.push('a');
                Self::mangle_type(&*r#type, mangled);
            }
        }
    }
}
