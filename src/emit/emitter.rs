#![allow(unused_parens)]

use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::check;
use crate::constants;
use crate::data::ast;
use crate::data::hir;
use crate::data::ir;
use crate::data::operand;
use crate::data::r#type;
use crate::hir;
use crate::util::symbol;

#[derive(Debug)]
pub struct Emitter<'env> {
    context: &'env check::Context,
    data: BTreeMap<symbol::Symbol, operand::Label>,
    functions: BTreeMap<symbol::Symbol, symbol::Symbol>,
}

impl<'env> Emitter<'env> {
    pub fn new(context: &'env check::Context) -> Self {
        Emitter {
            context,
            data: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }

    pub fn emit_unit(
        mut self,
        path: &std::path::Path,
        ast: &ast::Program,
    ) -> ir::Unit<hir::Function> {
        let mut functions = BTreeMap::new();

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
            statements.push(hir!(
                (MOVE
                    (self.emit_declaration(parameter, &mut variables))
                    (TEMP operand::Temporary::Argument(index)))
            ));
        }

        let name = self.mangle_function(function.name);
        let statement = self.emit_statement(&function.statements, &mut variables);

        statements.push(statement);

        hir::Function {
            name,
            statements: hir!((SEQ statements)),
        }
    }

    fn emit_expression(
        &mut self,
        expression: &ast::Expression,
        variables: &HashMap<symbol::Symbol, operand::Temporary>,
    ) -> hir::Tree {
        use ast::Expression::*;
        match expression {
            Boolean(false, _) => hir!((CONST 0)).into(),
            Boolean(true, _) => hir!((CONST 1)).into(),
            &Integer(integer, _) => hir!((CONST integer)).into(),

            &Character(character, _) => hir!((CONST character as i64)).into(),
            String(string, _) => {
                let symbol = symbol::intern(string);
                let label = *self
                    .data
                    .entry(symbol)
                    .or_insert_with(|| operand::Label::fresh("string"));

                hir!((NAME label)).into()
            }
            Variable(variable, _) => hir!((TEMP variables[variable])).into(),
            Array(expressions, _) => {
                let alloc = Self::emit_alloc(expressions.len());
                let base = hir!((TEMP operand::Temporary::fresh("array")));

                let mut statements = vec![
                    alloc,
                    hir!((MOVE (TEMP base.clone()) (TEMP operand::Temporary::Return(0)))),
                    hir!((MOVE (MEM (base.clone())) (CONST expressions.len() as i64))),
                ];

                use ir::Binary::Add;

                for (index, expression) in expressions.iter().enumerate() {
                    let expression = self.emit_expression(expression, variables).into();
                    statements.push(
                        hir!(
                        (MOVE
                            (MEM (Add (TEMP base.clone()) (CONST (index + 1) as i64 * constants::WORD_SIZE)))
                            expression)
                    ));
                }

                hir!(
                    (ESEQ
                        (SEQ statements)
                        (Add (TEMP base) (CONST constants::WORD_SIZE))))
                .into()
            }

            Binary(binary, left, right, _) => {
                let binary = ir::Binary::from(*binary);
                let left = self.emit_expression(left, variables);
                let right = self.emit_expression(right, variables);

                match binary {
                    #[rustfmt::skip]
                    ir::Binary::Mul
                    | ir::Binary::Hul
                    | ir::Binary::Mod
                    | ir::Binary::Div
                    | ir::Binary::Add
                    | ir::Binary::Sub => hir!((binary (left.into()) (right.into()))).into(),

                    ir::Binary::Lt
                    | ir::Binary::Le
                    | ir::Binary::Ge
                    | ir::Binary::Gt
                    | ir::Binary::Ne
                    | ir::Binary::Eq => hir::Tree::Condition(Box::new(move |r#true, r#false| {
                        hir!(
                            (CJUMP (binary (left.into()) (right.into())) r#true r#false)
                        )
                    })),

                    ir::Binary::And => hir::Tree::Condition(Box::new(move |r#true, r#false| {
                        let and = operand::Label::fresh("and");

                        hir!((SEQ
                            (hir::Condition::from(left)(and, r#false))
                            (LABEL and)
                            (hir::Condition::from(right)(r#true, r#false))
                        ))
                    })),
                    ir::Binary::Or => hir::Tree::Condition(Box::new(move |r#true, r#false| {
                        let or = operand::Label::fresh("or");

                        hir!((SEQ
                            (hir::Condition::from(left)(r#true, or))
                            (LABEL or)
                            (hir::Condition::from(right)(r#true, r#false))
                        ))
                    })),

                    ir::Binary::Xor | ir::Binary::Ls | ir::Binary::Rs | ir::Binary::ARs => {
                        unreachable!("[INTERNAL ERROR]: no XOR, LSHIFT, RSHIFT in AST")
                    }
                }
            }

            Unary(ast::Unary::Neg, expression, _) => {
                use ir::Binary::Sub;
                let expression = self.emit_expression(expression, variables).into();
                hir!((Sub (CONST 0) expression)).into()
            }
            Unary(ast::Unary::Not, expression, _) => {
                use ir::Binary::Xor;
                let expression = self.emit_expression(expression, variables).into();
                hir!((Xor (CONST 1) expression)).into()
            }

            Index(array, array_index, _) => {
                use ir::Binary::Add;
                use ir::Binary::Ge;
                use ir::Binary::Lt;
                use ir::Binary::Mul;
                use ir::Binary::Sub;

                let base = operand::Temporary::fresh("base");
                let index = operand::Temporary::fresh("index");

                let low = operand::Label::fresh("low");
                let high = operand::Label::fresh("high");
                let out = operand::Label::fresh("out");
                let r#in = operand::Label::fresh("in");

                hir!(
                    (ESEQ
                        (SEQ
                            (MOVE (TEMP base) (self.emit_expression(&*array, variables).into()))
                            (MOVE (TEMP index) (self.emit_expression(&*array_index, variables).into()))
                            (CJUMP (Lt (TEMP index) (CONST 0)) out low)
                            (LABEL low)
                            (CJUMP (Ge (TEMP index) (MEM (Sub (TEMP base) (CONST constants::WORD_SIZE)))) out high)
                            (LABEL high)
                            (JUMP (NAME r#in))
                            (LABEL out)
                            (SCALL (NAME (operand::Label::Fixed(symbol::intern_static(constants::XI_OUT_OF_BOUNDS)))))
                            (LABEL r#in))
                        (MEM (Add (TEMP base) (Mul (TEMP index) (CONST constants::WORD_SIZE)))))
                ).into()
            }
            Call(call) if symbol::resolve(call.name) == "length" => {
                use ir::Binary::Sub;
                let address = self.emit_expression(&call.arguments[0], variables).into();
                hir!((MEM (Sub address (CONST constants::WORD_SIZE)))).into()
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
            name: Box::new(hir::Expression::from(operand::Label::Fixed(
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
        hir!(
            (SCALL
                (NAME operand::Label::Fixed(symbol::intern_static(constants::XI_ALLOC)))
                (CONST (length + 1) as i64 * constants::WORD_SIZE))
        )
    }

    fn emit_declaration(
        &mut self,
        declaration: &ast::Declaration,
        variables: &mut HashMap<symbol::Symbol, operand::Temporary>,
    ) -> hir::Expression {
        let fresh = operand::Temporary::fresh("t");
        variables.insert(declaration.name, fresh);
        hir!((TEMP fresh))
    }

    fn emit_statement(
        &mut self,
        statement: &ast::Statement,
        variables: &mut HashMap<symbol::Symbol, operand::Temporary>,
    ) -> hir::Statement {
        use ast::Statement::*;
        match statement {
            #[rustfmt::skip]
            Assignment(left, right, _) => {
                hir!(
                    (MOVE
                        (self.emit_expression(left, variables).into())
                        (self.emit_expression(right, variables).into()))
                )
            }
            Call(call) => hir::Statement::Call(self.emit_call(call, variables)),
            Initialization(declarations, ast::Expression::Call(call), _) => {
                let mut statements = vec![hir::Statement::Call(self.emit_call(call, variables))];

                for (index, declaration) in declarations.iter().enumerate() {
                    if let Some(declaration) = declaration {
                        statements.push(hir!(
                            (MOVE
                                (self.emit_declaration(declaration, variables))
                                (TEMP operand::Temporary::Return(index)))
                        ));
                    }
                }

                hir::Statement::Sequence(statements)
            }
            #[rustfmt::skip]
            Initialization(declarations, expression, _) => {
                assert!(declarations.len() == 1 && declarations[0].is_some());
                let declaration = declarations[0].as_ref().unwrap();
                hir!(
                    (MOVE
                        (self.emit_declaration(declaration, variables))
                        (self.emit_expression(expression, variables).into()))
                )
            }
            Declaration(declaration, _) => {
                hir!(
                    (MOVE
                        (self.emit_declaration(declaration, variables))
                        (CONST 0))
                )
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

                hir!(
                    (SEQ
                        (hir::Condition::from(self.emit_expression(condition, variables))(r#true, r#false))
                        (LABEL r#true)
                        (self.emit_statement(r#if, variables))
                        (LABEL r#false))
                )
            }
            If(condition, r#if, Some(r#else), _) => {
                let r#true = operand::Label::fresh("true");
                let r#false = operand::Label::fresh("false");
                let endif = operand::Label::fresh("endif");

                hir!(
                    (SEQ
                        (hir::Condition::from(self.emit_expression(condition, variables))(r#true, r#false))
                        (LABEL r#true)
                        (self.emit_statement(r#if, variables))
                        (JUMP (NAME endif))
                        (LABEL r#false)
                        (self.emit_statement(r#else, variables))
                        (LABEL endif))
                )
            }
            While(condition, statements, _) => {
                let r#while = operand::Label::fresh("while");
                let r#true = operand::Label::fresh("true");
                let r#false = operand::Label::fresh("false");

                hir!(
                    (SEQ
                        (LABEL r#while)
                        (hir::Condition::from(self.emit_expression(condition, variables))(r#true, r#false))
                        (LABEL r#true)
                        (self.emit_statement(statements, variables))
                        (JUMP (NAME r#while))
                        (LABEL r#false))
                )
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
