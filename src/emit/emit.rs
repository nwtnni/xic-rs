#![allow(unused_parens)]

use crate::abi;
use crate::check;
use crate::check::Entry;
use crate::check::GlobalScope;
use crate::check::LocalScope;
use crate::check::Scope;
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
    classes: Map<Symbol, abi::class::Layout>,
    locals: Map<Symbol, Temporary>,
    data: Map<Symbol, Label>,
    bss: Map<Symbol, (ir::Visibility, usize)>,
}

pub fn emit_unit(
    path: &std::path::Path,
    context: &mut check::Context,
    ast: &ast::Program,
) -> ir::Unit<hir::Function> {
    let mut classes = Map::default();

    for class in context
        .class_implementations()
        .chain(context.class_signatures())
    {
        classes
            .entry(*class)
            .or_insert_with(|| abi::class::Layout::new(context, class));
    }

    let mut emitter = Emitter {
        classes,
        context,
        locals: Map::default(),
        data: Map::default(),
        bss: Map::default(),
    };

    let mut functions = Map::default();
    let mut initialize = Vec::new();

    for item in &ast.items {
        match item {
            ast::Item::Global(global) => {
                if let Some((name, function)) = emitter.emit_global(global) {
                    initialize.push(hir!((EXP (CALL (NAME (Label::Fixed(name))) 0))));
                    functions.insert(name, function);
                }
            }
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
            visibility: ir::Visibility::Local,
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
    fn emit_global(&mut self, global: &ast::Global) -> Option<(Symbol, hir::Function)> {
        let (name, statement) = match global {
            ast::Global::Declaration(declaration) => {
                let statement = self.emit_declaration(GlobalScope::Global, declaration)?;
                let name =
                    abi::mangle::global_initialization(declaration.iter().map(|(name, _)| {
                        (
                            &name.symbol,
                            self.get_variable(GlobalScope::Global, name).unwrap(),
                        )
                    }));
                (name, statement)
            }
            ast::Global::Initialization(initialization) => {
                let statement =
                    self.emit_initialization(Scope::Global(GlobalScope::Global), initialization);
                let name = abi::mangle::global_initialization(
                    initialization
                        .declarations
                        .iter()
                        .flatten()
                        .map(|declaration| {
                            (
                                &declaration.name.symbol,
                                self.get_variable(GlobalScope::Global, &declaration.name)
                                    .unwrap(),
                            )
                        }),
                );
                (name, statement)
            }
        };

        Some((
            name,
            hir::Function {
                name,
                statement,
                arguments: 0,
                returns: 0,
                visibility: ir::Visibility::Local,
            },
        ))
    }

    fn emit_class_initialization(&mut self, class: &Symbol) -> (Symbol, hir::Function) {
        let size = abi::mangle::class_size(class);

        // Reserve 1 word in BSS section for class size
        self.bss.insert(size, (ir::Visibility::Global, 1));

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
        let size_class = self.classes[class].field_len() as i64 * abi::WORD;
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
            (ir::Visibility::Global, self.classes[class].size()),
        );

        // Copy superclass virtual table
        if let Some(superclass) = superclass {
            let size_superclass = self.classes[&superclass].size();
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
                hir!((CJUMP (LT (TEMP offset) (CONST size_superclass as i64 * abi::WORD)) r#while r#done)),
                hir!((LABEL r#done)),
            ]);
        }

        // Selectively override superclass entries
        for (symbol, (_, entry)) in &self.context.get_class(class).unwrap().1 {
            let (parameters, returns) = match entry {
                Entry::Variable(_) | Entry::Signature(_, _) => continue,
                Entry::Function(parameters, returns) => (parameters, returns),
            };

            let offset = self.classes[class].method(symbol).unwrap();
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
                visibility: ir::Visibility::Global,
            },
        )
    }

    fn emit_function(
        &mut self,
        scope: GlobalScope,
        function: &ast::Function,
    ) -> (Symbol, hir::Function) {
        self.locals.clear();

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
        let argument_offset = match scope {
            GlobalScope::Global => 0,
            GlobalScope::Class(_) => 1,
        };

        for (index, parameter) in function.parameters.iter().enumerate() {
            #[rustfmt::skip]
            statements.push(hir!(
                (MOVE
                    (self.emit_single_declaration(Scope::Local, &parameter.name, &parameter.r#type))
                    (TEMP (Temporary::Argument(index + argument_offset))))
            ));
        }

        let (visibility, scope) = match scope {
            GlobalScope::Global => (ir::Visibility::Global, LocalScope::Function { returns }),
            GlobalScope::Class(class) => {
                (ir::Visibility::Local, LocalScope::Method { class, returns })
            }
        };

        self.context.push(scope);
        statements.push(self.emit_statement(&function.statements));
        self.context.pop();

        (
            name,
            hir::Function {
                name,
                statement: hir::Statement::Sequence(statements),
                arguments: function.parameters.len() + argument_offset,
                returns: function.returns.len(),
                visibility,
            },
        )
    }

    fn emit_statement(&mut self, statement: &ast::Statement) -> hir::Statement {
        use ast::Statement::*;
        match statement {
            #[rustfmt::skip]
            Assignment(left, right, _) => {
                hir!(
                    (MOVE
                        (self.emit_expression(left).into())
                        (self.emit_expression(right).into()))
                )
            }
            Call(call) => hir::Statement::Expression(self.emit_call(call)),
            Initialization(initialization) => {
                self.emit_initialization(Scope::Local, initialization)
            }
            Declaration(declaration, _) => match self.emit_declaration(Scope::Local, declaration) {
                // Note: relies on the parser and type-checker to ensure that we never assign
                // into a declaration that requires initialization (e.g. x: int[10]).
                None => hir!((EXP (CONST 0))),
                Some(statement) => statement,
            },
            Return(expressions, _) => hir::Statement::Return(
                expressions
                    .iter()
                    .map(|expression| self.emit_expression(expression).into())
                    .collect(),
            ),
            Sequence(statements, _) => hir::Statement::Sequence(
                statements
                    .iter()
                    .map(|statement| self.emit_statement(statement))
                    .collect(),
            ),
            If(condition, r#if, None, _) => {
                let r#true = Label::fresh("true");
                let r#false = Label::fresh("false");

                hir!(
                    (SEQ
                        (hir::Condition::from(self.emit_expression(condition))(r#true, r#false))
                        (LABEL r#true)
                        (self.emit_statement(r#if))
                        (LABEL r#false))
                )
            }
            If(condition, r#if, Some(r#else), _) => {
                let r#true = Label::fresh("true");
                let r#false = Label::fresh("false");
                let endif = Label::fresh("endif");

                hir!(
                    (SEQ
                        (hir::Condition::from(self.emit_expression(condition))(r#true, r#false))
                        (LABEL r#true)
                        (self.emit_statement(r#if))
                        (JUMP endif)
                        (LABEL r#false)
                        (self.emit_statement(r#else))
                        (LABEL endif))
                )
            }
            While(r#do, condition, statement, _) => {
                let r#while = Label::fresh("while");
                let r#true = Label::fresh("true");
                let r#false = Label::fresh("false");

                // Note: we negate `condition` here as an optimization, so that the main body of the
                // loop is the fallthrough (false) branch after the conditional jump. CFG destruction
                // tries to trace the false branch first, so this will keep the loop body closer in
                // the final assembly. IF we placed the loop body in the true branch, then we would
                // trace the rest of the program in the false branch before outputting the loop body.
                let condition = hir::Condition::from(
                    self.emit_expression(&condition.negate_logical()),
                )(r#true, r#false);

                self.context.push(LocalScope::While(Some(r#true)));
                let statement = self.emit_statement(statement);
                self.context.pop();

                match r#do {
                    ast::Do::Yes => {
                        hir!(
                            (SEQ
                                (LABEL r#while)
                                statement
                                condition
                                (LABEL r#false)
                                (JUMP r#while)
                                (LABEL r#true))
                        )
                    }
                    ast::Do::No => {
                        hir!(
                            (SEQ
                                (LABEL r#while)
                                condition
                                (LABEL r#false)
                                statement
                                (JUMP r#while)
                                (LABEL r#true))
                        )
                    }
                }
            }
            Break(_) => {
                let label = self.context.get_scoped_while().flatten().unwrap();
                hir!((JUMP label))
            }
        }
    }

    fn emit_expression(&self, expression: &ast::Expression) -> hir::Tree {
        use ast::Expression::*;
        match expression {
            Boolean(false, _) => hir!((CONST 0)).into(),
            Boolean(true, _) => hir!((CONST 1)).into(),
            &Integer(integer, _) => hir!((CONST integer)).into(),
            &Character(character, _) => hir!((CONST character as i64)).into(),
            String(string, span) => self.emit_expression(&ast::Expression::Array(
                string
                    .bytes()
                    .map(|byte| byte as i64)
                    .map(|byte| ast::Expression::Integer(byte, Span::default()))
                    .collect::<Vec<_>>(),
                *span,
            )),
            Null(_) => hir!((MEM (CONST 0))).into(),
            This(_) => hir!((TEMP Temporary::Argument(0))).into(),
            Super(_) => todo!(),
            Variable(variable) => {
                if let Some(temporary) = self.locals.get(&variable.symbol).copied() {
                    return hir!((TEMP temporary)).into();
                }

                if let Some(r#type) = self.get_variable(GlobalScope::Global, variable) {
                    let address = Label::Fixed(abi::mangle::global(&variable.symbol, r#type));
                    return hir!((MEM (NAME address))).into();
                }

                self.emit_class_field(
                    &self.context.get_scoped_class().unwrap(),
                    &ast::Expression::This(Span::default()),
                    &variable.symbol,
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
                    let expression = self.emit_expression(expression).into();
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
                let left = self.emit_expression(left);
                let right = self.emit_expression(right);

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
                let left = self.emit_expression(left);
                let right = self.emit_expression(right);

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
                let expression = self.emit_expression(expression).into();
                hir!((SUB (CONST 0) expression)).into()
            }
            Unary(ast::Unary::Not, expression, _) => {
                let expression = self.emit_expression(expression).into();
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
                            (MOVE (TEMP base) (self.emit_expression(&*array).into()))
                            (MOVE (TEMP index) (self.emit_expression(&*array_index).into()))
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
                let address = self.emit_expression(array).into();
                hir!((MEM (SUB address (CONST abi::WORD)))).into()
            }
            Dot(class, receiver, field, _) => self
                .emit_class_field(&class.get().unwrap(), receiver, &field.symbol)
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
            Call(call) => self.emit_call(call).into(),
        }
    }

    fn emit_call(&self, call: &ast::Call) -> hir::Expression {
        match &*call.function {
            ast::Expression::Variable(variable) => self
                .emit_function_call(variable, &call.arguments)
                .unwrap_or_else(|| {
                    self.emit_class_method_call(
                        &self.context.get_scoped_class().unwrap(),
                        &ast::Expression::This(Span::default()),
                        variable,
                        &call.arguments,
                    )
                }),
            ast::Expression::Dot(class, receiver, method, _) => self.emit_class_method_call(
                &class.get().unwrap(),
                receiver,
                method,
                &call.arguments,
            ),
            _ => unreachable!(),
        }
    }

    fn emit_function_call(
        &self,
        function: &ast::Identifier,
        arguments: &[ast::Expression],
    ) -> Option<hir::Expression> {
        let (parameters, returns) = self.get_signature(GlobalScope::Global, function)?;

        let function = abi::mangle::function(&function.symbol, parameters, returns);
        let returns = returns.len();
        let arguments = arguments
            .iter()
            .map(|argument| self.emit_expression(argument).into())
            .collect::<Vec<_>>();

        Some(hir::Expression::Call(
            Box::new(hir::Expression::from(Label::Fixed(function))),
            arguments,
            returns,
        ))
    }

    fn emit_class_method_call(
        &self,
        class: &Symbol,
        receiver: &ast::Expression,
        method: &ast::Identifier,
        arguments: &[ast::Expression],
    ) -> hir::Expression {
        let instance = Temporary::fresh("instance");
        let receiver = self.emit_expression(receiver).into();

        let mut arguments = arguments
            .iter()
            .map(|argument| self.emit_expression(argument).into())
            .collect::<Vec<_>>();
        let returns = self
            .get_signature(GlobalScope::Class(*class), method)
            .map(|(_, returns)| returns.len())
            .unwrap();

        arguments.insert(0, hir!((TEMP instance)));

        let method = hir!(
            (MEM (ADD
                (MEM (TEMP instance))
                (CONST self.classes[class].method(&method.symbol).unwrap() as i64 * abi::WORD)))
        );

        let call = hir::Expression::Call(Box::new(method), arguments, returns);

        hir!((ESEQ (MOVE (TEMP instance) receiver) call))
    }

    fn emit_class_field(
        &self,
        class: &Symbol,
        receiver: &ast::Expression,
        field: &Symbol,
    ) -> hir::Expression {
        let base = match self.context.get_superclass(class) {
            // 8-byte offset for virtual table pointer
            None => hir!((CONST abi::WORD)),
            Some(superclass) => {
                let superclass_size = Label::Fixed(abi::mangle::class_size(&superclass));
                hir!((MEM (NAME superclass_size)))
            }
        };

        let offset = self.classes[class]
            .field(field)
            .map(|index| hir!((CONST index as i64 * abi::WORD)))
            .expect("[TYPE ERROR]: unbound field in class");

        let receiver = self.emit_expression(receiver).into();

        hir!((MEM (ADD receiver (ADD base offset))))
    }

    fn emit_initialization(
        &mut self,
        scope: Scope,
        ast::Initialization {
            declarations,
            expression,
            ..
        }: &ast::Initialization,
    ) -> hir::Statement {
        #[rustfmt::skip]
        if let [Some(declaration)] = declarations.as_slice() {
            return hir!(
                (MOVE
                    (self.emit_single_declaration(scope, &declaration.name, &declaration.r#type))
                    (self.emit_expression(expression).into()))
            );
        };

        #[rustfmt::skip]
        let mut statements = vec![hir!(
            (EXP (self.emit_expression(expression).into()))
        )];

        for (index, declaration) in declarations.iter().enumerate() {
            if let Some(declaration) = declaration {
                #[rustfmt::skip]
                statements.push(hir!(
                    (MOVE
                        (self.emit_single_declaration(scope, &declaration.name, &declaration.r#type))
                        (TEMP (Temporary::Return(index))))
                ));
            }
        }

        hir::Statement::Sequence(statements)
    }

    fn emit_declaration<S: Into<Scope>>(
        &mut self,
        scope: S,
        declaration: &ast::Declaration,
    ) -> Option<hir::Statement> {
        let scope = scope.into();
        let statements = declaration
            .iter()
            .map(|(name, r#type)| self.emit_single_declaration(scope, name, r#type))
            .filter_map(|expression| match expression {
                hir::Expression::Sequence(statement, _) => Some(*statement),
                _ => None,
            })
            .collect::<Vec<_>>();

        match statements.is_empty() {
            true => None,
            false => Some(hir::Statement::Sequence(statements)),
        }
    }

    fn emit_single_declaration<S: Into<Scope>>(
        &mut self,
        scope: S,
        name: &ast::Identifier,
        r#type: &ast::Type,
    ) -> hir::Expression {
        let fresh = match scope.into() {
            Scope::Global(GlobalScope::Class(_)) => unreachable!(),
            Scope::Global(GlobalScope::Global) => {
                let r#type = self.get_variable(GlobalScope::Global, name).unwrap();
                let name = abi::mangle::global(&name.symbol, r#type);
                self.bss.insert(name, (ir::Visibility::Local, 1));
                hir!((MEM (NAME Label::Fixed(name))))
            }
            Scope::Local => {
                let fresh = Temporary::fresh(symbol::resolve(name.symbol));
                self.locals.insert(name.symbol, fresh);
                hir!((TEMP fresh))
            }
        };

        match r#type {
            ast::Type::Bool(_)
            | ast::Type::Int(_)
            | ast::Type::Class(_)
            | ast::Type::Array(_, None, _) => fresh,
            ast::Type::Array(r#type, Some(length), _) => {
                let mut lengths = Vec::new();
                let declaration = self.emit_array_declaration(r#type, length, &mut lengths);

                #[rustfmt::skip]
                lengths.push(hir!((MOVE (fresh.clone()) (declaration))));

                hir::Expression::Sequence(
                    Box::new(hir::Statement::Sequence(lengths)),
                    Box::new(fresh),
                )
            }
        }
    }

    fn emit_array_declaration(
        &mut self,
        r#type: &ast::Type,
        len: &ast::Expression,
        lengths: &mut Vec<hir::Statement>,
    ) -> hir::Expression {
        let length = Temporary::fresh("length");
        let array = Temporary::fresh("array");
        let alloc = Label::Fixed(symbol::intern_static(abi::XI_ALLOC));

        lengths.push(hir!((MOVE (TEMP length) (self.emit_expression(len).into()))));

        let mut statements = vec![
            hir!((MOVE (TEMP array)
                (CALL
                    (NAME alloc)
                    1
                    (MUL (ADD (TEMP length) (CONST 1)) (CONST abi::WORD))))),
            hir!((MOVE (MEM (TEMP array)) (TEMP length))),
        ];

        match r#type {
            ast::Type::Bool(_)
            | ast::Type::Int(_)
            | ast::Type::Array(_, None, _)
            | ast::Type::Class(_) => (),
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
                            (self.emit_array_declaration(r#type, len, lengths)))),
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

    fn get_signature(
        &self,
        scope: GlobalScope,
        name: &ast::Identifier,
    ) -> Option<(&[r#type::Expression], &[r#type::Expression])> {
        match self.context.get(scope, &name.symbol)? {
            check::Entry::Function(parameters, returns)
            | check::Entry::Signature(parameters, returns) => Some((parameters, returns)),
            check::Entry::Variable(_) => None,
        }
    }

    fn get_variable(
        &self,
        scope: GlobalScope,
        name: &ast::Identifier,
    ) -> Option<&r#type::Expression> {
        match self.context.get(scope, &name.symbol)? {
            check::Entry::Variable(r#type) => Some(r#type),
            check::Entry::Function(_, _) | check::Entry::Signature(_, _) => None,
        }
    }
}
