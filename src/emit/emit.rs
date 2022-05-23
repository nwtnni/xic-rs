#![allow(unused_parens)]

use crate::abi;
use crate::check;
use crate::check::Entry;
use crate::check::GlobalScope;
use crate::check::LocalScope;
use crate::data::ast;
use crate::data::hir;
use crate::data::ir;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::r#type;
use crate::data::span::Span;
use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::hir;
use crate::Map;

struct Emitter<'env> {
    context: &'env mut check::Context,
    r#virtual: abi::r#virtual::Table,
    data: Map<Symbol, Label>,
    bss: Map<Symbol, usize>,
}

pub fn emit_unit(
    path: &std::path::Path,
    context: &mut check::Context,
    ast: &ast::Program,
) -> ir::Unit<hir::Function> {
    let mut emitter = Emitter {
        r#virtual: abi::r#virtual::Table::new(context),
        context,
        data: Map::default(),
        bss: Map::default(),
    };

    let mut functions = Map::default();
    let mut initialize = Vec::new();

    for item in &ast.items {
        match item {
            ast::Item::Global(_) => todo!(),
            ast::Item::Class(class) => {
                for item in &class.items {
                    let method = match item {
                        ast::ClassItem::Field(_) => continue,
                        ast::ClassItem::Method(method) => method,
                    };

                    let (name, function) =
                        emitter.emit_function(GlobalScope::Class(class.name.symbol), method);

                    functions.insert(name, function);
                }
            }
            ast::Item::Function(function) => {
                let (name, function) = emitter.emit_function(GlobalScope::Global, function);
                functions.insert(name, function);
            }
        }
    }

    for class in emitter
        .context
        .class_implementations()
        .copied()
        .collect::<Vec<_>>()
    {
        let (name, function) = emitter.emit_class_initialization(&class);
        functions.insert(name, function);
        initialize.push(hir!((EXP (CALL (NAME Label::Fixed(name)) 0))));
    }

    initialize.push(hir!((RETURN)));
    functions.insert(
        abi::mangle::init(),
        hir::Function {
            name: abi::mangle::init(),
            statement: hir::Statement::Sequence(initialize),
            arguments: 0,
            returns: 0,
            global: false,
        },
    );

    ir::Unit {
        name: symbol::intern(path.to_string_lossy().trim_start_matches("./")),
        functions,
        data: emitter.data,
        bss: emitter.bss,
    }
}

impl<'env> Emitter<'env> {
    fn emit_class_initialization(&mut self, class: &Symbol) -> (Symbol, hir::Function) {
        let size = abi::mangle::class_size(class);

        // Reserve 1 word in BSS section for class size
        self.bss.insert(size, 1);

        let enter = Label::fresh("enter");
        let exit = Label::fresh("exit");

        let mut statements = vec![
            hir!((CJUMP (NE (MEM (NAME Label::Fixed(size))) (CONST 0)) exit enter)),
            hir!((LABEL enter)),
        ];

        let superclass = self.context.get_superclass(class);

        // Recursively initialize superclass
        if let Some(superclass) = superclass {
            let initialize = Label::Fixed(abi::mangle::class_initialization(&superclass));
            statements.push(hir!((EXP (CALL (NAME initialize) 0))));
        }

        // Initialize class size
        let size_class = self.r#virtual.field_len(class).unwrap() as i64 * abi::WORD;
        let size_superclass = superclass
            .map(|superclass| hir!((MEM (NAME Label::Fixed(abi::mangle::class_size(&superclass))))))
            .unwrap_or_else(|| hir!((CONST abi::WORD)));

        statements.push(
            hir!((MOVE (MEM (NAME Label::Fixed(size))) (ADD size_superclass (CONST size_class)))),
        );

        // Initialize class virtual table
        let virtual_table_class = abi::mangle::class_virtual_table(class);

        // Reserve n words in BSS section for class virtual table
        self.bss.insert(
            virtual_table_class,
            self.r#virtual.method_len(class).unwrap(),
        );

        // Copy superclass virtual table
        if let Some(superclass) = superclass {
            let length = self.r#virtual.method_len(&superclass).unwrap();
            let virtual_table_superclass =
                Label::Fixed(abi::mangle::class_virtual_table(&superclass));

            let offset = Temporary::fresh("offset");
            let r#while = Label::fresh("while");
            let done = Label::fresh("done");

            statements.extend([
                hir!((MOVE (TEMP offset) (CONST 0))),
                hir!((LABEL r#while)),
                hir!((MOVE
                    (MEM (ADD (NAME Label::Fixed(virtual_table_class)) (TEMP offset)))
                    (MEM (ADD (NAME virtual_table_superclass) (TEMP offset))))),
                hir!((MOVE (TEMP offset) (ADD (TEMP offset) (CONST abi::WORD)))),
                hir!((CJUMP (LT (TEMP offset) (CONST length as i64 * abi::WORD)) r#while r#done)),
                hir!((LABEL r#done)),
            ]);
        }

        // Selectively override superclass entries
        for (symbol, (_, entry)) in &self.context.get_class(class).unwrap().1 {
            let (parameters, returns) = match entry {
                Entry::Variable(_) | Entry::Signature(_, _) => continue,
                Entry::Function(parameters, returns) => (parameters, returns),
            };

            let offset = self.r#virtual.method(class, symbol).unwrap();
            let method = Label::Fixed(abi::mangle::method(class, symbol, parameters, returns));

            statements.push(hir!(
                (MOVE
                    (MEM (ADD (NAME Label::Fixed(virtual_table_class)) (CONST offset as i64 * abi::WORD)))
                    (NAME method))
            ));
        }

        statements.push(hir!((LABEL exit)));
        statements.push(hir!((RETURN)));

        let name = abi::mangle::class_initialization(class);

        (
            name,
            hir::Function {
                name,
                statement: hir::Statement::Sequence(statements),
                arguments: 0,
                returns: 0,
                global: false,
            },
        )
    }

    fn emit_function(
        &mut self,
        scope: GlobalScope,
        function: &ast::Function,
    ) -> (Symbol, hir::Function) {
        let mut variables = Map::default();
        let mut statements = Vec::new();

        let (parameters, returns) = self.get_signature(scope, &function.name).unwrap();

        let name = match scope {
            GlobalScope::Global => {
                abi::mangle::function(&function.name.symbol, parameters, returns)
            }
            GlobalScope::Class(class) => {
                abi::mangle::method(&class, &function.name.symbol, parameters, returns)
            }
        };

        let returns = returns.to_vec();
        let offset = match scope {
            GlobalScope::Global => 0,
            GlobalScope::Class(_) => 1,
        };

        for (index, parameter) in function.parameters.iter().enumerate() {
            #[rustfmt::skip]
            statements.push(hir!(
                (MOVE
                    (self.emit_declaration(parameter, &mut variables))
                    (TEMP (Temporary::Argument(index + offset))))
            ));
        }

        let scope = match scope {
            GlobalScope::Global => LocalScope::Function { returns },
            GlobalScope::Class(class) => LocalScope::Method { class, returns },
        };

        self.context.push(scope);
        statements.push(self.emit_statement(&function.statements, &mut variables));
        self.context.pop();

        (
            name,
            hir::Function {
                name,
                statement: hir::Statement::Sequence(statements),
                arguments: function.parameters.len(),
                returns: function.returns.len(),
                global: true,
            },
        )
    }

    fn emit_expression(
        &mut self,
        expression: &ast::Expression,
        variables: &Map<Symbol, Temporary>,
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
            Null(_) => hir!((MEM (CONST 0))).into(),
            This(_) => hir!((TEMP Temporary::Argument(0))).into(),
            Variable(variable) => {
                if let Some(temporary) = variables.get(&variable.symbol).copied() {
                    return hir!((TEMP temporary)).into();
                }

                if let Some(r#type) = self.get_variable(GlobalScope::Global, variable) {
                    let address = Label::Fixed(abi::mangle::global(&variable.symbol, r#type));
                    return hir!((MEM (NAME address))).into();
                }

                self.emit_field_access(
                    &self.context.get_scoped_class().unwrap(),
                    &ast::Expression::This(Span::default()),
                    &variable.symbol,
                    variables,
                )
                .into()
            }
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

                let array_left = Temporary::fresh("array");
                let array_right = Temporary::fresh("array");
                let array = Temporary::fresh("array");

                let length_left = Temporary::fresh("length");
                let length_right = Temporary::fresh("length");
                let length = Temporary::fresh("length");

                let alloc = Label::Fixed(symbol::intern_static(abi::XI_ALLOC));

                let while_left = Label::fresh("while");
                let done_left = Label::fresh("true");
                let bound_left = Temporary::fresh("bound");

                let while_right = Label::fresh("while");
                let done_right = Label::fresh("true");
                let bound_right = Temporary::fresh("bound");

                let address = Temporary::fresh("address");

                hir!(
                    (ESEQ
                        (SEQ
                            (MOVE (TEMP array_left) (left.into()))
                            (MOVE (TEMP length_left) (MEM (SUB (TEMP array_left) (CONST abi::WORD))))

                            (MOVE (TEMP array_right) (right.into()))
                            (MOVE (TEMP length_right) (MEM (SUB (TEMP array_right) (CONST abi::WORD))))

                            // Allocate destination with correct length (+1 for length metadata)
                            (MOVE (TEMP length) (ADD (TEMP length_left) (TEMP length_right)))
                            (MOVE
                                (TEMP array)
                                (CALL (NAME alloc) 1 (ADD (MUL (TEMP length) (CONST abi::WORD)) (CONST abi::WORD))))
                            (MOVE (MEM (TEMP array)) (TEMP length))
                            (MOVE (TEMP address) (ADD (TEMP array) (CONST abi::WORD)))

                            // Copy left array into final destination, starting at
                            // `array + WORD`
                            (MOVE (TEMP bound_left) (ADD (TEMP array_left) (MUL (TEMP length_left) (CONST abi::WORD))))
                            (CJUMP (AE (TEMP array_left) (TEMP bound_left)) done_left while_left)
                            (LABEL while_left)
                            (MOVE (MEM (TEMP address)) (MEM (TEMP array_left)))
                            (MOVE (TEMP array_left) (ADD (TEMP array_left) (CONST abi::WORD)))
                            (MOVE (TEMP address) (ADD (TEMP address) (CONST abi::WORD)))
                            (CJUMP (AE (TEMP array_left) (TEMP bound_left)) done_left while_left)
                            (LABEL done_left)

                            // Copy right array into final destination, starting at
                            // `array + WORD + length_left * WORD`
                            (MOVE (TEMP bound_right) (ADD (TEMP array_right) (MUL (TEMP length_right) (CONST abi::WORD))))
                            (CJUMP (AE (TEMP array_right) (TEMP bound_right)) done_right while_right)
                            (LABEL while_right)
                            (MOVE (MEM (TEMP address)) (MEM (TEMP array_right)))
                            (MOVE (TEMP array_right) (ADD (TEMP array_right) (CONST abi::WORD)))
                            (MOVE (TEMP address) (ADD (TEMP address) (CONST abi::WORD)))
                            (CJUMP (AE (TEMP array_right) (TEMP bound_right)) done_right while_right)
                            (LABEL done_right))
                        (ADD (TEMP array) (CONST abi::WORD)))
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
                            (hir::Statement::Return(vec![hir!((CONST 0)); self.context.get_scoped_returns().unwrap().len()]))
                            (LABEL r#in))
                        (MEM (ADD (TEMP base) (MUL (TEMP index) (CONST abi::WORD)))))
                ).into()
            }
            Length(array, _) => {
                let address = self.emit_expression(array, variables).into();
                hir!((MEM (SUB address (CONST abi::WORD)))).into()
            }
            Dot(class, receiver, field, _) => self
                .emit_field_access(&class.get().unwrap(), receiver, &field.symbol, variables)
                .into(),
            New(class, _) => {
                let xi_alloc = Label::Fixed(symbol::intern_static(abi::XI_ALLOC));
                let class_size = Label::Fixed(abi::mangle::class_size(&class.symbol));
                let class_virtual_table =
                    Label::Fixed(abi::mangle::class_virtual_table(&class.symbol));

                let new = Temporary::fresh("new");

                hir!(
                (ESEQ
                    (SEQ
                        (MOVE (TEMP new) (CALL (NAME xi_alloc) 1 (MEM (NAME class_size))))
                        (MOVE (MEM (TEMP new)) (NAME class_virtual_table)))
                    (TEMP new))
                )
                .into()
            }
            Call(call) => self.emit_call(call, variables).into(),
        }
    }

    fn emit_call(
        &mut self,
        call: &ast::Call,
        variables: &Map<Symbol, Temporary>,
    ) -> hir::Expression {
        let mut arguments = call
            .arguments
            .iter()
            .map(|argument| self.emit_expression(argument, variables).into())
            .collect::<Vec<_>>();

        let (class, receiver, method) = match &*call.function {
            ast::Expression::Variable(variable) => {
                if let Some((parameters, returns)) =
                    self.get_signature(GlobalScope::Global, variable)
                {
                    let function = abi::mangle::function(&variable.symbol, parameters, returns);
                    let returns = returns.len();
                    return hir::Expression::Call(
                        Box::new(hir::Expression::from(Label::Fixed(function))),
                        arguments,
                        returns,
                    );
                }

                let class = self.context.get_scoped_class().unwrap();
                let receiver = self
                    .emit_expression(&ast::Expression::This(Span::default()), variables)
                    .into();
                (class, receiver, variable)
            }
            ast::Expression::Dot(class, receiver, method, _) => {
                let class = class.get().unwrap();
                let receiver = self.emit_expression(receiver, variables).into();
                (class, receiver, method)
            }
            _ => unreachable!(),
        };

        let instance = Temporary::fresh("instance");
        let returns = self
            .get_signature(GlobalScope::Class(class), method)
            .map(|(_, returns)| returns.len())
            .unwrap();

        arguments.insert(0, hir!((TEMP instance)));

        let method = hir!(
            (MEM (ADD
                (MEM (TEMP instance))
                (CONST self.r#virtual.method(&class, &method.symbol).unwrap() as i64 * abi::WORD)))
        );

        let call = hir::Expression::Call(Box::new(method), arguments, returns);

        hir!((ESEQ (MOVE (TEMP instance) receiver) call))
    }

    fn emit_field_access(
        &mut self,
        class: &Symbol,
        receiver: &ast::Expression,
        field: &Symbol,
        variables: &Map<Symbol, Temporary>,
    ) -> hir::Expression {
        let base = match self.context.get_superclass(class) {
            // 8-byte offset for virtual table pointer
            None => hir!((CONST abi::WORD)),
            Some(superclass) => {
                let superclass_size = Label::Fixed(abi::mangle::class_size(&superclass));
                hir!((MEM (NAME superclass_size)))
            }
        };

        let offset = self
            .r#virtual
            .field(class, field)
            .map(|index| hir!((CONST index as i64 * abi::WORD)))
            .expect("[TYPE ERROR]: unbound field in class");

        let receiver = self.emit_expression(receiver, variables).into();

        hir!((MEM (ADD receiver (ADD base offset))))
    }

    fn emit_declaration(
        &mut self,
        declaration: &ast::SingleDeclaration,
        variables: &mut Map<Symbol, Temporary>,
    ) -> hir::Expression {
        let fresh = Temporary::fresh("t");
        variables.insert(declaration.name.symbol, fresh);
        match &*declaration.r#type {
            ast::Type::Bool(_)
            | ast::Type::Int(_)
            | ast::Type::Class(_)
            | ast::Type::Array(_, None, _) => {
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
        variables: &mut Map<Symbol, Temporary>,
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
            ast::Type::Class(_) => todo!(),
            ast::Type::Array(r#type, Some(len), _) => {
                let r#while = Label::fresh("while");
                let done = Label::fresh("done");
                let address = Temporary::fresh("address");
                let bound = Temporary::fresh("bound");

                statements.extend([
                    hir!((MOVE (TEMP address) (ADD (TEMP array) (CONST abi::WORD)))),
                    hir!((MOVE (TEMP bound) (ADD (TEMP address) (MUL (TEMP length) (CONST abi::WORD))))),
                    hir!((CJUMP (AE (TEMP address) (TEMP bound)) done r#while)),
                    hir!((LABEL r#while)),
                    hir!(
                        (MOVE
                            (MEM (TEMP address))
                            (self.emit_array_declaration(r#type, len, variables, lengths)))),
                    hir!((MOVE (TEMP address) (ADD (TEMP address) (CONST abi::WORD)))),
                    hir!((CJUMP (AE (TEMP address) (TEMP bound)) done r#while)),
                    hir!((LABEL done)),
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
        variables: &mut Map<Symbol, Temporary>,
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
            Initialization(ast::Initialization { declarations, expression, span: _ }) if declarations.len() == 1 => {
                let declaration = declarations[0].as_ref().unwrap();
                hir!(
                    (MOVE
                        (self.emit_declaration(declaration, variables))
                        (self.emit_expression(expression, variables).into()))
                )
            }
            Initialization(ast::Initialization {
                declarations,
                expression,
                span: _,
            }) => {
                let call = match &**expression {
                    ast::Expression::Call(call) => call,
                    _ => unreachable!("[TYPE ERROR]: multiple non-function initialization"),
                };

                let mut statements =
                    vec![hir::Statement::Expression(self.emit_call(call, variables))];

                for (index, declaration) in declarations.iter().enumerate() {
                    if let Some(declaration) = declaration {
                        #[rustfmt::skip]
                        statements.push(hir!(
                            (MOVE
                                (self.emit_declaration(declaration, variables))
                                (TEMP (Temporary::Return(index))))
                        ));
                    }
                }

                hir::Statement::Sequence(statements)
            }

            Declaration(declaration, _) => {
                let declaration = match &**declaration {
                    ast::Declaration::Multiple(_) => todo!(),
                    ast::Declaration::Single(declaration) => declaration,
                };

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
            Break(_) => todo!(),
        }
    }

    fn get_signature(
        &self,
        scope: GlobalScope,
        name: &ast::Identifier,
    ) -> Option<(&[r#type::Expression], &[r#type::Expression])> {
        match self.context.get(scope, &name.symbol) {
            Some(
                check::Entry::Function(parameters, returns)
                | check::Entry::Signature(parameters, returns),
            ) => Some((parameters, returns)),
            Some(check::Entry::Variable(_)) => None,
            None => unreachable!(),
        }
    }

    fn get_variable(
        &self,
        scope: GlobalScope,
        name: &ast::Identifier,
    ) -> Option<&r#type::Expression> {
        match self.context.get(scope, &name.symbol) {
            Some(check::Entry::Variable(r#type)) => Some(r#type),
            Some(check::Entry::Function(_, _) | check::Entry::Signature(_, _)) => None,
            None => unreachable!(),
        }
    }
}
