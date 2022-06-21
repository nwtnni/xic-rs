#![allow(unused_parens)]

use std::cell::Cell;
use std::iter;

use crate::abi;
use crate::abi::Abi;
use crate::check;
use crate::check::Entry;
use crate::check::GlobalScope;
use crate::check::LocalScope;
use crate::check::Scope;
use crate::data::ast;
use crate::data::hir;
use crate::data::ir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::r#type;
use crate::data::span::Span;
use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::emit::library;
use crate::hir;
use crate::util;
use crate::Map;

// FIXME: take `Program<r#type::Expression>` as argument
pub fn emit_hir(
    context: &mut check::Context,
    path: &std::path::Path,
    abi: Abi,
    ast: &ast::Program<()>,
) -> ir::Unit<hir::Function> {
    log::info!(
        "[{}] Emitting HIR for {}...",
        std::any::type_name::<ast::Program<()>>(),
        path.display()
    );
    util::time!(
        "[{}] Done emitting HIR for {}",
        std::any::type_name::<ast::Program<()>>(),
        path.display()
    );

    let mut layouts = Map::default();

    for class in context
        .class_implementations()
        .chain(context.class_signatures())
    {
        layouts
            .entry(*class)
            .or_insert_with(|| abi::class::Layout::new(context, abi, class));
    }

    let mut emitter = Emitter {
        layouts,
        context,
        locals: Map::default(),
        data: Map::default(),
        bss: Map::default(),
        statics: Map::default(),
        out_of_bounds: Cell::new(None),
    };

    let mut functions = Map::default();
    let mut initialize = Vec::new();

    // Note: class initialization functions must be emitted before
    // global initialization functions, which can rely on the former.
    for item in &ast.items {
        let class = match item {
            ast::Item::Class(class) => class,
            _ => continue,
        };

        let (name, function) = emitter.emit_class_initialization(class);
        functions.insert(name, function);
        initialize.push(hir!((EXP (CALL (NAME name) 0))));
    }

    for item in &ast.items {
        match item {
            ast::Item::Global(global) => {
                if let Some((name, function)) = emitter.emit_global(global) {
                    functions.insert(name, function);
                    initialize.push(hir!((EXP (CALL (NAME name) 0))));
                }
            }
            ast::Item::Class(class) => {
                for item in &class.items {
                    let method = match item {
                        ast::ClassItem::Field(_) => continue,
                        ast::ClassItem::Method(method) => method,
                    };

                    let linkage = match (
                        emitter.layouts[&class.name.symbol].method_index(&method.name.symbol),
                        class.provenance.is_empty(),
                        method.declared.get(),
                    ) {
                        (None, true, false) => ir::Linkage::Local,
                        (None, true, true) => ir::Linkage::Global,
                        (Some(_), true, _) => ir::Linkage::Local,
                        (_, false, _) => ir::Linkage::LinkOnceOdr,
                    };

                    let (name, function) = emitter.emit_function(
                        GlobalScope::Class(class.name.symbol),
                        method,
                        linkage,
                    );

                    functions.insert(name, function);
                }
            }
            ast::Item::ClassTemplate(_) => unreachable!(),
            ast::Item::Function(function) => {
                let linkage = match (function.provenance.is_empty(), function.declared.get()) {
                    (true, false) => ir::Linkage::Local,
                    (true, true) => ir::Linkage::Global,
                    (false, _) => ir::Linkage::LinkOnceOdr,
                };

                let (name, function) =
                    emitter.emit_function(GlobalScope::Global, function, linkage);
                functions.insert(name, function);
            }
            ast::Item::FunctionTemplate(_) => unreachable!(),
        }
    }

    initialize.push(hir!((RETURN)));
    functions.insert(
        symbol::intern_static(abi::XI_INIT),
        hir::Function {
            name: symbol::intern_static(abi::XI_INIT),
            statement: hir::Statement::Sequence(initialize),
            arguments: 0,
            returns: 0,
            linkage: ir::Linkage::Local,
        },
    );

    let memdup = library::emit_memdup();
    functions.insert(memdup.name, memdup);

    let concat = library::emit_concat();
    functions.insert(concat.name, concat);

    ir::Unit {
        name: symbol::intern(path.to_string_lossy().trim_start_matches("./")),
        functions,
        data: emitter.data,
        bss: emitter.bss,
    }
}

struct Emitter<'env> {
    context: &'env mut check::Context,
    layouts: Map<Symbol, abi::class::Layout>,
    locals: Map<Symbol, Temporary>,
    data: Map<Label, Vec<Immediate>>,
    bss: Map<Symbol, (ir::Linkage, usize)>,
    statics: Map<Vec<Immediate>, Label>,
    out_of_bounds: Cell<Option<Label>>,
}

impl<'env> Emitter<'env> {
    fn emit_global(&mut self, global: &ast::Global<()>) -> Option<(Symbol, hir::Function)> {
        self.locals.clear();
        self.out_of_bounds.take();

        let (name, statement) = match global {
            ast::Global::Declaration(declaration) => {
                // Note: we don't need to push a `LocalScope::Function` as of now because
                // emitting IR for declarations can't read from or write to the local scope.
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
                if let ast::Expression::Integer(integer, _) = &*initialization.expression {
                    assert_eq!(initialization.declarations.len(), 1);
                    let name = initialization.declarations[0].as_ref().unwrap().name.symbol;
                    let r#type = r#type::Expression::Integer;
                    self.data.insert(
                        Label::Fixed(abi::mangle::global(&name, &r#type)),
                        vec![Immediate::Integer(*integer)],
                    );
                    return None;
                }

                self.context.push(LocalScope::Function {
                    returns: Vec::new(),
                });
                let statement =
                    self.emit_initialization(Scope::Global(GlobalScope::Global), initialization);
                self.context.pop();

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
                linkage: ir::Linkage::Local,
            },
        ))
    }

    fn emit_class_initialization(&mut self, class: &ast::Class<()>) -> (Symbol, hir::Function) {
        let size = abi::mangle::class_size(&class.name.symbol);

        let enter = Label::fresh("enter");
        let exit = Label::fresh("exit");

        let mut statements = vec![
            hir!((CJUMP (NE (MEM (NAME size)) (CONST 0)) exit enter)),
            hir!((LABEL enter)),
        ];

        // Recursively initialize superclass
        if let Some(superclass) = self.context.get_superclass(&class.name.symbol) {
            let initialize = abi::mangle::class_initialization(&superclass);
            statements.push(hir!((EXP (CALL (NAME initialize) 0))));
        }

        let linkage = match (class.provenance.is_empty(), class.declared.get()) {
            (true, false) => ir::Linkage::Local,
            (true, true) => ir::Linkage::Global,
            (false, _) => ir::Linkage::LinkOnceOdr,
        };

        self.emit_class_size(&class.name.symbol, linkage, &mut statements);
        self.emit_class_virtual_table(&class.name.symbol, linkage, &mut statements);

        statements.push(hir!((LABEL exit)));
        statements.push(hir!((RETURN)));

        let name = abi::mangle::class_initialization(&class.name.symbol);

        (
            name,
            hir::Function {
                name,
                statement: hir::Statement::Sequence(statements),
                arguments: 0,
                returns: 0,
                linkage,
            },
        )
    }

    fn emit_class_size(
        &mut self,
        class: &Symbol,
        linkage: ir::Linkage,
        statements: &mut Vec<hir::Statement>,
    ) {
        let size = abi::mangle::class_size(class);

        // Reserve 1 word in BSS section for class size
        self.bss.insert(size, (linkage, 1));

        let fields = self.layouts[class].field_len();

        #[rustfmt::skip]
        let interface = self.layouts[class]
            .interface()
            .map(|superclass| hir!((MEM (NAME abi::mangle::class_size(&superclass)))))
            .unwrap_or_else(|| hir!((CONST 0)));

        statements.push(hir!(
            (MOVE (MEM (NAME size)) (ADD interface (CONST fields as i64 * abi::WORD)))
        ));
    }

    fn emit_class_virtual_table(
        &mut self,
        class: &Symbol,
        linkage: ir::Linkage,
        statements: &mut Vec<hir::Statement>,
    ) {
        let virtual_table_class = abi::mangle::class_virtual_table(class);
        let virtual_table_class_size = match self.layouts[class].virtual_table_len() {
            Some(size) => size,
            None => return,
        };

        // Reserve n words in BSS section for class virtual table
        self.bss
            .insert(virtual_table_class, (linkage, virtual_table_class_size));

        // Copy superclass virtual table
        if let Some(superclass) = self.context.get_superclass(class) {
            let virtual_table_superclass = abi::mangle::class_virtual_table(&superclass);
            let virtual_table_superclass_size = self.layouts[&superclass]
                .virtual_table_len()
                .expect("[TYPE ERROR]: non-`final` class must have virtual table");

            let offset = Temporary::fresh("offset");
            let r#while = Label::fresh("while");
            let done = Label::fresh("done");

            statements.extend([
                hir!((MOVE (TEMP offset) (CONST 0))),
                hir!((LABEL r#while)),
                hir!((MOVE
                    (MEM (ADD (NAME virtual_table_class) (TEMP offset)))
                    (MEM (ADD (NAME virtual_table_superclass) (TEMP offset))))),
                hir!((MOVE (TEMP offset) (ADD (TEMP offset) (CONST abi::WORD)))),
                hir!((CJUMP (LT (TEMP offset) (CONST virtual_table_superclass_size as i64 * abi::WORD)) r#while r#done)),
                hir!((LABEL r#done)),
            ]);
        }

        // Selectively override superclass entries
        for (identifier, entry) in self.context.get_class(class).unwrap() {
            let (parameters, returns) = match entry {
                Entry::Function(parameters, returns) => (parameters, returns),
                Entry::Variable(_) | Entry::Signature(_, _) => continue,
            };

            let index = match self.layouts[class].method_index(&identifier.symbol) {
                Some(index) => index,
                None => continue,
            };

            let method = abi::mangle::method(class, &identifier.symbol, parameters, returns);

            statements.push(hir!(
                (MOVE
                    (MEM (ADD (NAME virtual_table_class) (CONST index as i64 * abi::WORD)))
                    (NAME method))
            ));
        }
    }

    fn emit_function(
        &mut self,
        scope: GlobalScope,
        function: &ast::Function<()>,
        linkage: ir::Linkage,
    ) -> (Symbol, hir::Function) {
        self.locals.clear();
        self.out_of_bounds.take();

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

        let scope = match scope {
            GlobalScope::Global => LocalScope::Function { returns },
            GlobalScope::Class(class) => LocalScope::Method { class, returns },
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
                linkage: if name == symbol::intern_static(abi::XI_MAIN) {
                    ir::Linkage::Global
                } else {
                    linkage
                },
            },
        )
    }

    fn emit_statement(&mut self, statement: &ast::Statement<()>) -> hir::Statement {
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

    fn emit_expression(&mut self, expression: &ast::Expression<()>) -> hir::Tree {
        use ast::Expression::*;
        match expression {
            Boolean(false, _) => hir!((CONST 0)).into(),
            Boolean(true, _) => hir!((CONST 1)).into(),
            &Integer(integer, _) => hir!((CONST integer)).into(),
            &Character(character, _) => hir!((CONST character as i64)).into(),
            String(string, _) => self.emit_expression(&ast::Expression::Array(
                string
                    .bytes()
                    .map(|byte| byte as i64)
                    .map(|integer| ast::Expression::Integer(integer, Span::default()))
                    .collect(),
                (),
                Span::default(),
            )),
            Null(_, _) => hir!((CONST 0)).into(),
            This(_, _) | Super(_, _) => hir!((TEMP Temporary::Argument(0))).into(),
            Variable(variable, _) => {
                assert!(variable.generics.is_none());

                if let Some(temporary) = self.locals.get(&variable.name.symbol).copied() {
                    return hir!((TEMP temporary)).into();
                }

                if let Some(r#type) = self.get_variable(GlobalScope::Global, &variable.name) {
                    return hir!((MEM (NAME abi::mangle::global(&variable.name.symbol, r#type))))
                        .into();
                }

                self.emit_class_field(
                    &self.context.get_scoped_class().unwrap(),
                    &ast::Expression::This((), Span::default()),
                    &variable.name.symbol,
                )
                .into()
            }
            Array(expressions, _, _) => {
                if let Some(label) = self.emit_static_array(expressions) {
                    return hir!((ADD (CALL (NAME abi::XI_MEMDUP) 1 (NAME label)) (CONST abi::WORD))).into();
                }

                let array = hir!((TEMP Temporary::fresh("array")));

                let mut statements = vec![
                    hir!(
                        (MOVE
                            (TEMP array.clone())
                            (CALL (NAME abi::XI_ALLOC) 1 (CONST (expressions.len() + 1) as i64 * abi::WORD)))
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
            Binary(binary, left, right, _, _) if binary.get() == ast::Binary::Cat => {
                let left = self.emit_expression(left);
                let right = self.emit_expression(right);
                hir!((CALL (NAME abi::XI_CONCAT) 1 (left.into()) (right.into()))).into()
            }
            Binary(binary, left, right, _, _) => {
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
                        hir!((binary(left.into())(right.into()))).into()
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

            Unary(ast::Unary::Neg, expression, _, _) => {
                let expression = self.emit_expression(expression).into();
                hir!((SUB (CONST 0) expression)).into()
            }
            Unary(ast::Unary::Not, expression, _, _) => {
                let expression = self.emit_expression(expression).into();
                hir!((XOR (CONST 1) expression)).into()
            }

            Index(array, array_index, _, _) => {
                let base = Temporary::fresh("base");
                let index = Temporary::fresh("index");

                let r#in = Label::fresh("in");

                let mut statements = vec![
                    hir!((MOVE (TEMP base) (self.emit_expression(&*array).into()))),
                    hir!((MOVE (TEMP index) (self.emit_expression(&*array_index).into()))),
                ];

                // Ensure we only emit one out of bounds block per function
                match self.out_of_bounds.get() {
                    Some(out) => {
                        statements.extend([
                            hir!((CJUMP (AE (TEMP index) (MEM (SUB (TEMP base) (CONST abi::WORD)))) out r#in)),
                        ]);
                    }
                    None => {
                        let out = Label::fresh("out");
                        self.out_of_bounds.set(Some(out));

                        statements.extend([
                            hir!((CJUMP (AE (TEMP index) (MEM (SUB (TEMP base) (CONST abi::WORD)))) out r#in)),
                            hir!((LABEL out)),
                            hir!((EXP (CALL (NAME abi::XI_OUT_OF_BOUNDS) 0))),
                            // In order to (1) minimize special logic for `XI_OUT_OF_BOUNDS` and (2) still
                            // treat it correctly in dataflow analyses as an exit site, we put this dummy
                            // return statement here.
                            //
                            // The number of returns must match the rest of the function, so return values
                            // are defined along all paths to the exit.
                            hir!((hir::Statement::Return(vec![hir!((CONST 0)); self.context.get_scoped_returns().unwrap().len()]))),
                        ]);
                    }
                };

                statements.push(hir!((LABEL r#in)));

                hir!(
                    (ESEQ
                        (hir::Statement::Sequence(statements))
                        (MEM (ADD (TEMP base) (MUL (TEMP index) (CONST abi::WORD)))))
                )
                .into()
            }
            Length(array, _) => {
                let address = self.emit_expression(array).into();
                hir!((MEM (SUB address (CONST abi::WORD)))).into()
            }
            Dot(class, receiver, field, _, _) => self
                .emit_class_field(&class.get().unwrap(), receiver, &field.symbol)
                .into(),
            New(variable, _) => {
                assert!(variable.generics.is_none());

                let class_size = abi::mangle::class_size(&variable.name.symbol);

                match self.layouts[&variable.name.symbol].virtual_table_len() {
                    None => hir!((CALL (NAME abi::XI_ALLOC) 1 (MEM (NAME class_size)))).into(),
                    Some(_) => {
                        let new = Temporary::fresh("new");
                        let virtual_table = abi::mangle::class_virtual_table(&variable.name.symbol);
                        hir!(
                            (ESEQ
                                (SEQ
                                    (MOVE (TEMP new) (CALL (NAME abi::XI_ALLOC) 1 (MEM (NAME class_size))))
                                    (MOVE (MEM (TEMP new)) (NAME virtual_table)))
                                (TEMP new))
                        )
                        .into()
                    }
                }
            }
            Call(call) => self.emit_call(call).into(),
        }
    }

    fn emit_call(&mut self, call: &ast::Call<()>) -> hir::Expression {
        match &*call.function {
            ast::Expression::Variable(variable, _) => {
                assert!(variable.generics.is_none());

                self.emit_function_call(variable, &call.arguments)
                    .unwrap_or_else(|| {
                        self.emit_class_method_call(
                            &self.context.get_scoped_class().unwrap(),
                            &ast::Expression::This((), Span::default()),
                            &variable.name,
                            &call.arguments,
                        )
                    })
            }
            ast::Expression::Dot(class, receiver, method, _, _) => self.emit_class_method_call(
                &class.get().unwrap(),
                receiver,
                method,
                &call.arguments,
            ),
            _ => unreachable!(),
        }
    }

    fn emit_function_call(
        &mut self,
        function: &ast::Variable<()>,
        arguments: &[ast::Expression<()>],
    ) -> Option<hir::Expression> {
        let (parameters, returns) = self.get_signature(GlobalScope::Global, &function.name)?;

        let function = abi::mangle::function(&function.name.symbol, parameters, returns);
        let returns = returns.len();
        let arguments = arguments
            .iter()
            .map(|argument| self.emit_expression(argument).into())
            .collect::<Vec<_>>();

        Some(hir::Expression::Call(
            Box::new(hir::Expression::from(function)),
            arguments,
            returns,
        ))
    }

    fn emit_class_method_call(
        &mut self,
        class: &Symbol,
        receiver: &ast::Expression<()>,
        method: &ast::Identifier,
        arguments: &[ast::Expression<()>],
    ) -> hir::Expression {
        let instance = Temporary::fresh("instance");

        let mut arguments = arguments
            .iter()
            .map(|argument| self.emit_expression(argument).into())
            .collect::<Vec<_>>();
        let returns = self
            .get_signature(GlobalScope::Class(*class), method)
            .map(|(_, returns)| returns.len())
            .unwrap();

        arguments.insert(0, hir!((TEMP instance)));

        let method = match self.layouts[class].method_index(&method.symbol) {
            // Special case: if `method_index` returns `None`, then this method
            // does not have a virtual table entry (see `crate::abi::class`
            // for details), and we should call the method as a static function.
            None => {
                let (parameters, returns) = self
                    .get_signature(GlobalScope::Class(*class), method)
                    .expect("[TYPE ERROR]: unbound method");

                hir!((NAME abi::mangle::method(class, &method.symbol, parameters, returns)))
            }
            // Special case: if the receiver is `super`, then we know its virtual
            // table statically, since we want to force the superclass
            // implementation even if it is overridden by subclasses.
            Some(index) if matches!(receiver, ast::Expression::Super(_, _)) => {
                hir!(
                    (MEM (ADD
                        (NAME abi::mangle::class_virtual_table(class))
                        (CONST index as i64 * abi::WORD))
                ))
            }
            Some(index) => {
                hir!((MEM (ADD (MEM (TEMP instance)) (CONST index as i64 * abi::WORD))))
            }
        };

        hir!(
            (ESEQ
                (MOVE (TEMP instance) (self.emit_expression(receiver).into()))
                (hir::Expression::Call(Box::new(method), arguments, returns)))
        )
    }

    fn emit_class_field(
        &mut self,
        class: &Symbol,
        receiver: &ast::Expression<()>,
        field: &Symbol,
    ) -> hir::Expression {
        #[rustfmt::skip]
        let base = self.layouts[class]
            .interface()
            .map(|superclass| hir!((MEM (NAME abi::mangle::class_size(&superclass)))))
            .unwrap_or_else(|| hir!((CONST 0)));

        let index = self.layouts[class]
            .field_index(class, field)
            .expect("[TYPE ERROR]: unbound field in class");

        hir!((MEM (ADD (self.emit_expression(receiver).into()) (ADD base (CONST index as i64 * abi::WORD)))))
    }

    fn emit_initialization(
        &mut self,
        scope: Scope,
        ast::Initialization {
            declarations,
            expression,
            ..
        }: &ast::Initialization<()>,
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
        declaration: &ast::Declaration<()>,
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
        r#type: &ast::Type<()>,
    ) -> hir::Expression {
        let fresh = match scope.into() {
            Scope::Global(GlobalScope::Class(_)) => unreachable!(),
            Scope::Global(GlobalScope::Global) => {
                let r#type = self.get_variable(GlobalScope::Global, name).unwrap();
                let name = abi::mangle::global(&name.symbol, r#type);
                self.bss.insert(name, (ir::Linkage::Local, 1));
                hir!((MEM (NAME name)))
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
        r#type: &ast::Type<()>,
        len: &ast::Expression<()>,
        lengths: &mut Vec<hir::Statement>,
    ) -> hir::Expression {
        let length = Temporary::fresh("length");
        let array = Temporary::fresh("array");

        lengths.push(hir!((MOVE (TEMP length) (self.emit_expression(len).into()))));

        let mut statements = vec![
            hir!((MOVE (TEMP array) (CALL (NAME abi::XI_ALLOC) 1 (MUL (ADD (TEMP length) (CONST 1)) (CONST abi::WORD))))),
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

    fn emit_static_array(&mut self, expressions: &[ast::Expression<()>]) -> Option<Label> {
        let array = iter::once(expressions.len() as i64)
            .map(Option::Some)
            .chain(expressions.iter().map(|expression| match expression {
                ast::Expression::Integer(integer, _) => Some(*integer),
                ast::Expression::Character(character, _) => Some(*character as i64),
                _ => None,
            }))
            .map(|integer| integer.map(Immediate::Integer))
            .collect::<Option<Vec<_>>>()?;

        // Duplicate already exists
        if let Some(label) = self.statics.get(&array) {
            return Some(*label);
        }

        let label = Label::fresh("array");
        self.statics.insert(array.clone(), label);
        self.data.insert(label, array);
        Some(label)
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
