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
use crate::data::symbol;
use crate::hir;

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
            #[rustfmt::skip]
            statements.push(hir!(
                (MOVE
                    (self.emit_declaration(parameter, &mut variables))
                    (hir::Expression::Argument(index)))
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
                let array = hir!((TEMP operand::Temporary::fresh("array")));

                let mut statements = vec![
                    hir!(
                        (MOVE
                            (TEMP array.clone())
                            (CALL
                                (NAME operand::Label::Fixed(symbol::intern_static(constants::XI_ALLOC)))
                                1
                                (CONST (expressions.len() + 1) as i64 * constants::WORD_SIZE)))
                    ),
                    hir!((MOVE (MEM (array.clone())) (CONST expressions.len() as i64))),
                ];

                use ir::Binary::Add;

                for (index, expression) in expressions.iter().enumerate() {
                    let expression = self.emit_expression(expression, variables).into();
                    statements.push(
                        hir!(
                        (MOVE
                            (MEM (Add (TEMP array.clone()) (CONST (index + 1) as i64 * constants::WORD_SIZE)))
                            expression)
                    ));
                }

                hir!(
                    (ESEQ
                        (SEQ statements)
                        (Add (TEMP array) (CONST constants::WORD_SIZE))))
                .into()
            }
            Binary(binary, left, right, _) if binary.get() == ast::Binary::Cat => {
                let left = self.emit_expression(left, variables);
                let right = self.emit_expression(right, variables);

                let address_left = operand::Temporary::fresh("array");
                let address_right = operand::Temporary::fresh("array");
                let address = operand::Temporary::fresh("array");

                let length_left = operand::Temporary::fresh("length");
                let length_right = operand::Temporary::fresh("length");
                let length = operand::Temporary::fresh("length");

                let alloc = operand::Label::Fixed(symbol::intern_static(constants::XI_ALLOC));

                let while_left = operand::Label::fresh("while");
                let true_left = operand::Label::fresh("true");
                let false_left = operand::Label::fresh("false");

                let while_right = operand::Label::fresh("while");
                let true_right = operand::Label::fresh("true");
                let false_right = operand::Label::fresh("false");

                let index = operand::Temporary::fresh("index");

                use ir::Binary::Add;
                use ir::Binary::Mul;
                use ir::Binary::Sub;

                use ir::Condition::Lt;

                hir!(
                    (ESEQ
                        (SEQ
                            (MOVE (TEMP address_left) (left.into()))
                            (MOVE (TEMP length_left) (MEM (Sub (TEMP address_left) (CONST constants::WORD_SIZE))))

                            (MOVE (TEMP address_right) (right.into()))
                            (MOVE (TEMP length_right) (MEM (Sub (TEMP address_right) (CONST constants::WORD_SIZE))))

                            // Allocate destination with correct length (+1 for length metadata)
                            (MOVE (TEMP length) (Add (TEMP length_left) (TEMP length_right)))
                            (MOVE
                                (TEMP address)
                                (CALL (NAME alloc) 1 (Mul (Add (TEMP length) (CONST 1)) (CONST constants::WORD_SIZE))))
                            (MOVE (MEM (TEMP address)) (TEMP length))
                            (MOVE (TEMP index) (CONST 1))

                            // Copy left array into final destination, starting at
                            // `address + WORD_SIZE`
                            (LABEL while_left)
                            (CJUMP (Lt (TEMP index) (Add (TEMP length_left) (CONST 1))) true_left false_left)
                            (LABEL true_left)
                            (MOVE
                                (MEM (Add (TEMP address) (Mul (TEMP index) (CONST constants::WORD_SIZE))))
                                (MEM (Add (TEMP address_left) (Mul (Sub (TEMP index) (CONST 1)) (CONST constants::WORD_SIZE)))))
                            (MOVE (TEMP index) (Add (TEMP index) (CONST 1)))
                            (JUMP while_left)
                            (LABEL false_left)

                            // Copy right array into final destination, starting at
                            // `address + WORD_SIZE + length_left * WORD_SIZE`
                            (LABEL while_right)
                            (CJUMP (Lt (TEMP index) (Add (TEMP length) (CONST 1))) true_right false_right)
                            (LABEL true_right)
                            (MOVE
                                (MEM (Add (TEMP address) (Mul (TEMP index) (CONST constants::WORD_SIZE))))
                                (MEM (Add (TEMP address_right) (Mul (Sub (Sub (TEMP index) (TEMP length_left)) (CONST 1)) (CONST constants::WORD_SIZE)))))
                            (MOVE (TEMP index) (Add (TEMP index) (CONST 1)))
                            (JUMP while_right)
                            (LABEL false_right))
                        (Add (TEMP address) (CONST constants::WORD_SIZE)))
                )
                .into()
            }
            Binary(binary, left, right, _) => {
                let left = self.emit_expression(left, variables);
                let right = self.emit_expression(right, variables);

                match binary.get() {
                    ast::Binary::Cat => unreachable!(),
                    ast::Binary::Mul
                    | ast::Binary::Hul
                    | ast::Binary::Mod
                    | ast::Binary::Div
                    | ast::Binary::Add
                    | ast::Binary::Sub => {
                        let binary = ir::Binary::from(binary.get());
                        hir!((binary (left.into()) (right.into()))).into()
                    }

                    ast::Binary::Lt
                    | ast::Binary::Le
                    | ast::Binary::Ge
                    | ast::Binary::Gt
                    | ast::Binary::Ne
                    | ast::Binary::Eq => {
                        let condition = ir::Condition::from(binary.get());
                        hir::Tree::Condition(Box::new(move |r#true, r#false| {
                            hir!(
                                (CJUMP (condition (left.into()) (right.into())) r#true r#false)
                            )
                        }))
                    }

                    ast::Binary::And => hir::Tree::Condition(Box::new(move |r#true, r#false| {
                        let and = operand::Label::fresh("and");

                        hir!((SEQ
                            (hir::Condition::from(left)(and, r#false))
                            (LABEL and)
                            (hir::Condition::from(right)(r#true, r#false))
                        ))
                    })),
                    ast::Binary::Or => hir::Tree::Condition(Box::new(move |r#true, r#false| {
                        let or = operand::Label::fresh("or");

                        hir!((SEQ
                            (hir::Condition::from(left)(r#true, or))
                            (LABEL or)
                            (hir::Condition::from(right)(r#true, r#false))
                        ))
                    })),
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
                use ir::Binary::Mul;
                use ir::Binary::Sub;

                use ir::Condition::Ge;
                use ir::Condition::Lt;

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
                            (JUMP r#in)
                            (LABEL out)
                            (EXP (CALL (NAME (operand::Label::Fixed(symbol::intern_static(constants::XI_OUT_OF_BOUNDS)))) 0))
                            (LABEL r#in))
                        (MEM (Add (TEMP base) (Mul (TEMP index) (CONST constants::WORD_SIZE)))))
                ).into()
            }
            Call(call) if symbol::resolve(call.name) == "length" => {
                use ir::Binary::Sub;
                let address = self.emit_expression(&call.arguments[0], variables).into();
                hir!((MEM (Sub address (CONST constants::WORD_SIZE)))).into()
            }
            Call(call) => self.emit_call(call, variables).into(),
        }
    }

    fn emit_call(
        &mut self,
        call: &ast::Call,
        variables: &HashMap<symbol::Symbol, operand::Temporary>,
    ) -> hir::Expression {
        hir::Expression::Call(
            Box::new(hir::Expression::from(operand::Label::Fixed(
                self.mangle_function(call.name),
            ))),
            call.arguments
                .iter()
                .map(|argument| self.emit_expression(argument, variables).into())
                .collect(),
            self.get_returns(call.name),
        )
    }

    fn emit_declaration(
        &mut self,
        declaration: &ast::Declaration,
        variables: &mut HashMap<symbol::Symbol, operand::Temporary>,
    ) -> hir::Expression {
        let fresh = operand::Temporary::fresh("t");
        variables.insert(declaration.name, fresh);
        match &declaration.r#type {
            ast::Type::Bool(_) | ast::Type::Int(_) | ast::Type::Array(_, None, _) => {
                hir!((TEMP fresh))
            }
            ast::Type::Array(r#type, Some(length), _) => {
                let mut lengths = Vec::new();
                let declaration =
                    self.emit_array_declaration(r#type, length, variables, &mut lengths);
                lengths.push(hir!((MOVE (TEMP fresh) (declaration))));

                hir::Expression::Sequence(
                    Box::new(hir!((SEQ lengths))),
                    Box::new(hir!((TEMP fresh))),
                )
            }
        }
    }

    fn emit_array_declaration(
        &mut self,
        r#type: &ast::Type,
        len: &ast::Expression,
        variables: &mut HashMap<symbol::Symbol, operand::Temporary>,
        lengths: &mut Vec<hir::Statement>,
    ) -> hir::Expression {
        let length = operand::Temporary::fresh("length");
        let array = operand::Temporary::fresh("array");
        let alloc = operand::Label::Fixed(symbol::intern_static(constants::XI_ALLOC));

        use ir::Binary::Add;
        use ir::Binary::Mul;

        lengths.push(hir!((MOVE (TEMP length) (self.emit_expression(len, variables).into()))));

        let mut statements = vec![
            hir!((MOVE (TEMP array)
                (CALL
                    (NAME alloc)
                    1
                    (Mul (Add (TEMP length) (CONST 1)) (CONST constants::WORD_SIZE))))),
            hir!((MOVE (MEM (TEMP array)) (TEMP length))),
        ];

        match r#type {
            ast::Type::Bool(_) | ast::Type::Int(_) | ast::Type::Array(_, None, _) => (),
            ast::Type::Array(r#type, Some(len), _) => {
                let r#while = operand::Label::fresh("while");
                let r#true = operand::Label::fresh("true");
                let r#false = operand::Label::fresh("false");
                let index = operand::Temporary::fresh("index");

                use ir::Condition::Lt;

                statements.extend([
                    hir!((MOVE (TEMP index) (CONST 0))),
                    hir!((LABEL r#while)),
                    hir!((CJUMP (Lt (TEMP index) (TEMP length)) r#true r#false)),
                    hir!((LABEL r#true)),
                    hir!(
                        (MOVE
                            (MEM (Add (TEMP array) (Mul (Add (TEMP index) (CONST 1)) (CONST constants::WORD_SIZE))))
                            (self.emit_array_declaration(r#type, len, variables, lengths)))),
                    hir!((MOVE (TEMP index) (Add (TEMP index) (CONST 1)))),
                    hir!((JUMP r#while)),
                    hir!((LABEL r#false)),
                ]);
            }
        }

        hir!((ESEQ (SEQ statements) (Add (TEMP array) (CONST constants::WORD_SIZE))))
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
            Call(call) => hir::Statement::Expression(self.emit_call(call, variables)),
            #[rustfmt::skip]
            Initialization(declarations, expression, _) if declarations.len() == 1 => {
                let declaration = declarations[0].as_ref().unwrap();
                hir!(
                    (MOVE
                        (self.emit_declaration(declaration, variables))
                        (self.emit_expression(expression, variables).into()))
                )
            }
            Initialization(declarations, ast::Expression::Call(call), _) => {
                let mut statements =
                    vec![hir::Statement::Expression(self.emit_call(call, variables))];

                for (index, declaration) in declarations.iter().enumerate() {
                    if let Some(declaration) = declaration {
                        #[rustfmt::skip]
                        statements.push(hir!(
                            (MOVE
                                (self.emit_declaration(declaration, variables))
                                (hir::Expression::Return(index)))
                        ));
                    }
                }

                hir::Statement::Sequence(statements)
            }
            Initialization(_, _, _) => {
                unreachable!("[TYPE ERROR]: multiple non-function initialization")
            }

            Declaration(declaration, _) => {
                hir::Statement::Expression(self.emit_declaration(declaration, variables))
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
                        (JUMP endif)
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
                        (JUMP r#while)
                        (LABEL r#false))
                )
            }
        }
    }

    fn get_returns(&self, name: symbol::Symbol) -> usize {
        match self.context.get(name) {
            Some(check::Entry::Function(_, returns))
            | Some(check::Entry::Signature(_, returns)) => returns.len(),
            _ => panic!("[INTERNAL ERROR]: type checking failed"),
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
