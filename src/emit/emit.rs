#![allow(unused_parens)]

use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::abi;
use crate::check;
use crate::data::ast;
use crate::data::hir;
use crate::data::ir;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::hir;

#[derive(Debug)]
struct Emitter<'env> {
    context: &'env check::Context,
    data: BTreeMap<Symbol, Label>,
    mangled: BTreeMap<Symbol, Symbol>,
    returns: usize,
}

pub fn emit_unit(
    path: &std::path::Path,
    context: &check::Context,
    ast: &ast::Program,
) -> ir::Unit<hir::Function> {
    let mut emitter = Emitter {
        context,
        data: BTreeMap::new(),
        mangled: BTreeMap::new(),
        returns: 0,
    };

    let mut functions = BTreeMap::new();

    for fun in &ast.functions {
        emitter.returns = emitter.get_returns(fun.name);
        let name = emitter.mangle_function(fun.name);
        let hir = emitter.emit_function(fun);
        functions.insert(name, hir);
    }

    ir::Unit {
        name: symbol::intern(path.to_string_lossy().trim_start_matches("./")),
        functions,
        data: emitter.data,
    }
}

impl<'env> Emitter<'env> {
    fn emit_function(&mut self, function: &ast::Function) -> hir::Function {
        let mut variables = HashMap::default();
        let mut statements = Vec::new();

        for (index, parameter) in function.parameters.iter().enumerate() {
            #[rustfmt::skip]
            statements.push(hir!(
                (MOVE
                    (self.emit_declaration(parameter, &mut variables))
                    (_ARG index))
            ));
        }

        let name = self.mangle_function(function.name);
        let statement = self.emit_statement(&function.statements, &mut variables);

        statements.push(statement);

        hir::Function {
            name,
            statement: hir::Statement::Sequence(statements),
            arguments: function.parameters.len(),
            returns: function.returns.len(),
        }
    }

    fn emit_expression(
        &mut self,
        expression: &ast::Expression,
        variables: &HashMap<Symbol, Temporary>,
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
                    .or_insert_with(|| Label::fresh("string"));

                hir!((NAME label)).into()
            }
            Variable(variable, _) => hir!((TEMP variables[variable])).into(),
            Array(expressions, _) => {
                let array = hir!((TEMP Temporary::fresh("array")));

                let mut statements = vec![
                    hir!(
                        (MOVE
                            (TEMP array.clone())
                            (CALL
                                (NAME Label::Fixed(symbol::intern_static(abi::XI_ALLOC)))
                                1
                                (CONST (expressions.len() + 1) as i64 * abi::WORD)))
                    ),
                    hir!((MOVE (MEM (array.clone())) (CONST expressions.len() as i64))),
                ];

                for (index, expression) in expressions.iter().enumerate() {
                    let expression = self.emit_expression(expression, variables).into();
                    statements.push(hir!(
                        (MOVE
                            (MEM (ADD (TEMP array.clone()) (CONST (index + 1) as i64 * abi::WORD)))
                            expression)
                    ));
                }

                hir!(
                    (ESEQ
                        (hir::Statement::Sequence(statements))
                        (ADD (TEMP array) (CONST abi::WORD))))
                .into()
            }
            Binary(binary, left, right, _) if binary.get() == ast::Binary::Cat => {
                let left = self.emit_expression(left, variables);
                let right = self.emit_expression(right, variables);

                let address_left = Temporary::fresh("array");
                let address_right = Temporary::fresh("array");
                let address = Temporary::fresh("array");

                let length_left = Temporary::fresh("length");
                let length_right = Temporary::fresh("length");
                let length = Temporary::fresh("length");

                let alloc = Label::Fixed(symbol::intern_static(abi::XI_ALLOC));

                let while_left = Label::fresh("while");
                let true_left = Label::fresh("true");
                let false_left = Label::fresh("false");

                let while_right = Label::fresh("while");
                let true_right = Label::fresh("true");
                let false_right = Label::fresh("false");

                let index = Temporary::fresh("index");

                hir!(
                    (ESEQ
                        (SEQ
                            (MOVE (TEMP address_left) (left.into()))
                            (MOVE (TEMP length_left) (MEM (SUB (TEMP address_left) (CONST abi::WORD))))

                            (MOVE (TEMP address_right) (right.into()))
                            (MOVE (TEMP length_right) (MEM (SUB (TEMP address_right) (CONST abi::WORD))))

                            // Allocate destination with correct length (+1 for length metadata)
                            (MOVE (TEMP length) (ADD (TEMP length_left) (TEMP length_right)))
                            (MOVE
                                (TEMP address)
                                (CALL (NAME alloc) 1 (MUL (ADD (TEMP length) (CONST 1)) (CONST abi::WORD))))
                            (MOVE (MEM (TEMP address)) (TEMP length))
                            (MOVE (TEMP index) (CONST 1))

                            // Copy left array into final destination, starting at
                            // `address + WORD`
                            (LABEL while_left)
                            (CJUMP (GE (TEMP index) (ADD (TEMP length_left) (CONST 1))) true_left false_left)
                            (LABEL false_left)
                            (MOVE
                                (MEM (ADD (TEMP address) (MUL (TEMP index) (CONST abi::WORD))))
                                (MEM (ADD (TEMP address_left) (MUL (SUB (TEMP index) (CONST 1)) (CONST abi::WORD)))))
                            (MOVE (TEMP index) (ADD (TEMP index) (CONST 1)))
                            (JUMP while_left)
                            (LABEL true_left)

                            // Copy right array into final destination, starting at
                            // `address + WORD + length_left * WORD`
                            (LABEL while_right)
                            (CJUMP (GE (TEMP index) (ADD (TEMP length) (CONST 1))) true_right false_right)
                            (LABEL false_right)
                            (MOVE
                                (MEM (ADD (TEMP address) (MUL (TEMP index) (CONST abi::WORD))))
                                (MEM (ADD (TEMP address_right) (MUL (SUB (SUB (TEMP index) (TEMP length_left)) (CONST 1)) (CONST abi::WORD)))))
                            (MOVE (TEMP index) (ADD (TEMP index) (CONST 1)))
                            (JUMP while_right)
                            (LABEL true_right))
                        (ADD (TEMP address) (CONST abi::WORD)))
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
                        let and = Label::fresh("and");

                        hir!((SEQ
                            (hir::Condition::from(left)(and, r#false))
                            (LABEL and)
                            (hir::Condition::from(right)(r#true, r#false))
                        ))
                    })),
                    ast::Binary::Or => hir::Tree::Condition(Box::new(move |r#true, r#false| {
                        let or = Label::fresh("or");

                        hir!((SEQ
                            (hir::Condition::from(left)(r#true, or))
                            (LABEL or)
                            (hir::Condition::from(right)(r#true, r#false))
                        ))
                    })),
                }
            }

            Unary(ast::Unary::Neg, expression, _) => {
                let expression = self.emit_expression(expression, variables).into();
                hir!((SUB (CONST 0) expression)).into()
            }
            Unary(ast::Unary::Not, expression, _) => {
                let expression = self.emit_expression(expression, variables).into();
                hir!((XOR (CONST 1) expression)).into()
            }

            Index(array, array_index, _) => {
                let base = Temporary::fresh("base");
                let index = Temporary::fresh("index");

                let high = Label::fresh("high");
                let out = Label::fresh("out");
                let r#in = Label::fresh("in");

                hir!(
                    (ESEQ
                        (SEQ
                            (MOVE (TEMP base) (self.emit_expression(&*array, variables).into()))
                            (MOVE (TEMP index) (self.emit_expression(&*array_index, variables).into()))
                            (CJUMP (AE (TEMP index) (MEM (SUB (TEMP base) (CONST abi::WORD)))) out high)
                            (LABEL high)
                            (JUMP r#in)
                            (LABEL out)
                            (EXP (CALL (NAME (Label::Fixed(symbol::intern_static(abi::XI_OUT_OF_BOUNDS)))) 0))
                            // In order to (1) minimize special logic for `XI_OUT_OF_BOUNDS` and (2) still
                            // treat it correctly in dataflow analyses as an exit site, we put this dummy
                            // return statement here.
                            //
                            // The number of returns must match the rest of the function, so return values
                            // are defined along all paths to the exit.
                            (hir::Statement::Return(vec![hir!((CONST 0)); self.returns]))
                            (LABEL r#in))
                        (MEM (ADD (TEMP base) (MUL (TEMP index) (CONST abi::WORD)))))
                ).into()
            }
            Call(call) if symbol::resolve(call.name) == "length" => {
                let address = self.emit_expression(&call.arguments[0], variables).into();
                hir!((MEM (SUB address (CONST abi::WORD)))).into()
            }
            Call(call) => self.emit_call(call, variables).into(),
        }
    }

    fn emit_call(
        &mut self,
        call: &ast::Call,
        variables: &HashMap<Symbol, Temporary>,
    ) -> hir::Expression {
        hir::Expression::Call(
            Box::new(hir::Expression::from(Label::Fixed(
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
        variables: &mut HashMap<Symbol, Temporary>,
    ) -> hir::Expression {
        let fresh = Temporary::fresh("t");
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
                    Box::new(hir::Statement::Sequence(lengths)),
                    Box::new(hir!((TEMP fresh))),
                )
            }
        }
    }

    fn emit_array_declaration(
        &mut self,
        r#type: &ast::Type,
        len: &ast::Expression,
        variables: &mut HashMap<Symbol, Temporary>,
        lengths: &mut Vec<hir::Statement>,
    ) -> hir::Expression {
        let length = Temporary::fresh("length");
        let array = Temporary::fresh("array");
        let alloc = Label::Fixed(symbol::intern_static(abi::XI_ALLOC));

        lengths.push(hir!((MOVE (TEMP length) (self.emit_expression(len, variables).into()))));

        let mut statements = vec![
            hir!((MOVE (TEMP array)
                (CALL
                    (NAME alloc)
                    1
                    (MUL (ADD (TEMP length) (CONST 1)) (CONST abi::WORD))))),
            hir!((MOVE (MEM (TEMP array)) (TEMP length))),
        ];

        match r#type {
            ast::Type::Bool(_) | ast::Type::Int(_) | ast::Type::Array(_, None, _) => (),
            ast::Type::Array(r#type, Some(len), _) => {
                let r#while = Label::fresh("while");
                let r#true = Label::fresh("true");
                let r#false = Label::fresh("false");
                let index = Temporary::fresh("index");

                statements.extend([
                    hir!((MOVE (TEMP index) (CONST 0))),
                    hir!((LABEL r#while)),
                    hir!((CJUMP (GE (TEMP index) (TEMP length)) r#true r#false)),
                    hir!((LABEL r#false)),
                    hir!(
                        (MOVE
                            (MEM (ADD (TEMP array) (MUL (ADD (TEMP index) (CONST 1)) (CONST abi::WORD))))
                            (self.emit_array_declaration(r#type, len, variables, lengths)))),
                    hir!((MOVE (TEMP index) (ADD (TEMP index) (CONST 1)))),
                    hir!((JUMP r#while)),
                    hir!((LABEL r#true)),
                ]);
            }
        }

        hir!(
            (ESEQ
                (hir::Statement::Sequence(statements))
                (ADD (TEMP array) (CONST abi::WORD)))
        )
    }

    fn emit_statement(
        &mut self,
        statement: &ast::Statement,
        variables: &mut HashMap<Symbol, Temporary>,
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
                                (_RET index))
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
                let r#true = Label::fresh("true");
                let r#false = Label::fresh("false");

                hir!(
                    (SEQ
                        (hir::Condition::from(self.emit_expression(condition, variables))(r#true, r#false))
                        (LABEL r#true)
                        (self.emit_statement(r#if, variables))
                        (LABEL r#false))
                )
            }
            If(condition, r#if, Some(r#else), _) => {
                let r#true = Label::fresh("true");
                let r#false = Label::fresh("false");
                let endif = Label::fresh("endif");

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
            While(r#do, condition, statements, _) => {
                let r#while = Label::fresh("while");
                let r#true = Label::fresh("true");
                let r#false = Label::fresh("false");

                let condition = match hir::Condition::from(
                    self.emit_expression(condition, variables),
                )(r#true, r#false)
                {
                    hir::Statement::CJump {
                        condition,
                        left,
                        right,
                        r#true,
                        r#false,
                    } => hir::Statement::CJump {
                        condition: condition.negate(),
                        left,
                        right,
                        r#true,
                        r#false,
                    },
                    _ => unreachable!(),
                };

                match r#do {
                    ast::Do::Yes => {
                        hir!(
                            (SEQ
                                (LABEL r#while)
                                (self.emit_statement(statements, variables))
                                (condition)
                                (LABEL r#false)
                                (JUMP r#while)
                                (LABEL r#true))
                        )
                    }
                    ast::Do::No => {
                        hir!(
                            (SEQ
                                (LABEL r#while)
                                (condition)
                                (LABEL r#false)
                                (self.emit_statement(statements, variables))
                                (JUMP r#while)
                                (LABEL r#true))
                        )
                    }
                }
            }
        }
    }

    fn get_returns(&self, name: Symbol) -> usize {
        match self.context.get(name) {
            Some(check::Entry::Function(_, returns))
            | Some(check::Entry::Signature(_, returns)) => returns.len(),
            _ => panic!("[INTERNAL ERROR]: type checking failed"),
        }
    }

    fn mangle_function(&mut self, name: Symbol) -> Symbol {
        if let Some(mangled) = self.mangled.get(&name) {
            return *mangled;
        }

        let (parameters, returns) = match self.context.get(name) {
            Some(check::Entry::Function(parameters, returns))
            | Some(check::Entry::Signature(parameters, returns)) => (parameters, returns),
            _ => panic!("[INTERNAL ERROR]: type checking failed"),
        };

        let mangled = symbol::intern(abi::mangle_function(
            symbol::resolve(name),
            parameters,
            returns,
        ));

        self.mangled.insert(name, mangled);
        mangled
    }
}
