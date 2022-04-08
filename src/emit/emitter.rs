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
            let argument = hir::temporary(operand::Temporary::Argument(index));
            let declaration = self.emit_declaration(parameter, &mut variables);
            statements.push(hir::r#move(declaration, argument));
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
            Boolean(false, _) => hir::integer(0).into(),
            Boolean(true, _) => hir::integer(1).into(),
            Integer(integer, _) => hir::integer(*integer).into(),
            Character(character, _) => hir::integer(*character as i64).into(),
            String(string, _) => {
                let symbol = symbol::intern(string);
                let label = *self
                    .data
                    .entry(symbol)
                    .or_insert_with(|| operand::Label::fresh("string"));

                hir::label(label).into()
            }
            Variable(variable, _) => hir::temporary(variables[variable]).into(),
            Array(expressions, _) => {
                let alloc = Self::emit_alloc(expressions.len());
                let base = hir::temporary(operand::Temporary::fresh("array"));

                let mut statements = vec![
                    alloc,
                    hir::r#move(base.clone(), hir::temporary(operand::Temporary::Return(0))),
                    hir::r#move(hir::memory(base.clone()), expressions.len() as i64),
                ];

                for (index, expression) in expressions.iter().enumerate() {
                    let expression = self.emit_expression(expression, variables);
                    let address = hir::memory(hir::binary(
                        ir::Binary::Add,
                        base.clone(),
                        (index + 1) as i64 * constants::WORD_SIZE,
                    ));
                    statements.push(hir::r#move(address, expression));
                }

                hir::Expression::Sequence(
                    Box::new(hir::Statement::Sequence(statements)),
                    Box::new(hir::binary(ir::Binary::Add, base, constants::WORD_SIZE)),
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
                    | ir::Binary::Sub => hir::binary(binary, left, right).into(),

                    ir::Binary::Lt
                    | ir::Binary::Le
                    | ir::Binary::Ge
                    | ir::Binary::Gt
                    | ir::Binary::Ne
                    | ir::Binary::Eq => hir::Tree::Condition(Box::new(move |r#true, r#false| {
                        hir::Statement::CJump(hir::binary(binary, left, right), r#true, r#false)
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

            Unary(ast::Unary::Neg, expression, _) => hir::binary(
                ir::Binary::Sub,
                0,
                self.emit_expression(expression, variables),
            )
            .into(),
            Unary(ast::Unary::Not, expression, _) => hir::binary(
                ir::Binary::Xor,
                1,
                self.emit_expression(expression, variables),
            )
            .into(),

            Index(array, array_index, _) => {
                let base = operand::Temporary::fresh("base");
                let index = operand::Temporary::fresh("index");

                let length = hir::memory(hir::binary(ir::Binary::Sub, base, constants::WORD_SIZE));
                let address = hir::binary(
                    ir::Binary::Add,
                    base,
                    hir::binary(ir::Binary::Mul, index, constants::WORD_SIZE),
                );

                let low = operand::Label::fresh("low");
                let high = operand::Label::fresh("high");
                let out = operand::Label::fresh("out");
                let r#in = operand::Label::fresh("in");

                let bounds = hir::Statement::Sequence(vec![
                    hir::r#move(base, self.emit_expression(&*array, variables)),
                    hir::r#move(index, self.emit_expression(&*array_index, variables)),
                    hir::Statement::CJump(hir::binary(ir::Binary::Lt, index, 0), out, low),
                    hir::Statement::Label(low),
                    hir::Statement::CJump(hir::binary(ir::Binary::Ge, index, length), out, high),
                    hir::Statement::Label(high),
                    hir::Statement::Jump(hir::label(r#in)),
                    hir::Statement::Label(out),
                    hir::Statement::Call(hir::Call {
                        name: Box::new(hir::label(operand::Label::Fixed(symbol::intern(
                            constants::XI_OUT_OF_BOUNDS,
                        )))),
                        arguments: Vec::new(),
                    }),
                    hir::Statement::Label(r#in),
                ]);

                hir::Expression::Sequence(Box::new(bounds), Box::new(address)).into()
            }
            Call(call) if call.name == symbol::intern("length") => {
                let address = self.emit_expression(&call.arguments[0], variables);
                hir::memory(hir::binary(ir::Binary::Sub, address, constants::WORD_SIZE)).into()
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
            name: Box::new(hir::label(operand::Label::Fixed(
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
        let alloc = hir::label(operand::Label::Fixed(symbol::intern(constants::XI_ALLOC)));
        let bytes = hir::integer((length + 1) as i64 * constants::WORD_SIZE);
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
        hir::temporary(fresh)
    }

    fn emit_statement(
        &mut self,
        statement: &ast::Statement,
        variables: &mut HashMap<symbol::Symbol, operand::Temporary>,
    ) -> hir::Statement {
        use ast::Statement::*;
        match statement {
            Assignment(left, right, _) => {
                let lhs = self.emit_expression(left, variables);
                let rhs = self.emit_expression(right, variables);
                hir::r#move(lhs, rhs)
            }
            Call(call) => hir::Statement::Call(self.emit_call(call, variables)),
            Initialization(declarations, ast::Expression::Call(call), _) => {
                let mut statements = vec![hir::Statement::Call(self.emit_call(call, variables))];

                for (index, declaration) in declarations.iter().enumerate() {
                    if let Some(declaration) = declaration {
                        let variable = self.emit_declaration(declaration, variables);
                        let r#return = hir::temporary(operand::Temporary::Return(index));
                        statements.push(hir::r#move(variable, r#return));
                    }
                }

                hir::Statement::Sequence(statements)
            }
            Initialization(declarations, expression, _) => {
                assert!(declarations.len() == 1 && declarations[0].is_some());
                let declaration = declarations[0].as_ref().unwrap();
                let variable = self.emit_declaration(declaration, variables);
                let expression = self.emit_expression(expression, variables);
                hir::r#move(variable, expression)
            }
            Declaration(declaration, _) => {
                hir::r#move(self.emit_declaration(declaration, variables), 0)
            }
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
