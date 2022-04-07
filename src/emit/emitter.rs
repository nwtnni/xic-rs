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
    env: &'env check::Context,
    data: HashMap<symbol::Symbol, operand::Label>,
    funs: HashMap<symbol::Symbol, symbol::Symbol>,
}

impl<'env> Emitter<'env> {
    pub fn new(env: &'env check::Context) -> Self {
        Emitter {
            env,
            data: HashMap::new(),
            funs: HashMap::new(),
        }
    }

    pub fn emit_unit(mut self, path: &std::path::Path, ast: &ast::Program) -> ir::Unit<hir::Fun> {
        let mut funs = HashMap::with_capacity(ast.functions.len());
        for fun in &ast.functions {
            let id = self.mangle_fun(fun.name);
            let ir = self.emit_fun(fun);
            funs.insert(id, ir);
        }
        ir::Unit {
            name: symbol::intern(path.to_string_lossy()),
            funs,
            data: self.data,
        }
    }

    fn emit_fun(&mut self, fun: &ast::Function) -> hir::Fun {
        let mut vars = HashMap::default();
        let mut seq = Vec::new();
        for (i, arg) in fun.parameters.iter().enumerate() {
            let reg = hir::Exp::Temp(operand::Temp::Arg(i));
            let dec = self.emit_dec(arg, &mut vars);
            seq.push(hir::Stm::Move(dec, reg));
        }
        let name = self.mangle_fun(fun.name);
        let body = self.emit_stm(&fun.body, &mut vars);
        seq.push(body);
        hir::Fun {
            name,
            body: hir::Stm::Seq(seq),
        }
    }

    fn emit_exp(
        &mut self,
        exp: &ast::Expression,
        vars: &HashMap<symbol::Symbol, operand::Temp>,
    ) -> hir::Tree {
        use ast::Expression::*;
        match exp {
            Boolean(false, _) => hir::Exp::Int(0).into(),
            Boolean(true, _) => hir::Exp::Int(1).into(),
            Integer(i, _) => hir::Exp::Int(*i).into(),
            Character(c, _) => hir::Exp::Int(*c as i64).into(),
            String(s, _) => {
                let symbol = symbol::intern(s);
                let label = *self
                    .data
                    .entry(symbol)
                    .or_insert_with(|| operand::Label::new("STR"));
                hir::Exp::Name(label).into()
            }
            Variable(v, _) => hir::Exp::Temp(vars[v]).into(),
            Array(vs, _) => {
                let alloc = Self::emit_alloc(vs.len());
                let base = hir::Exp::Temp(operand::Temp::new("ARR"));

                let mut seq = Vec::with_capacity(vs.len() + 2);
                seq.push(alloc);
                seq.push(hir::Stm::Move(
                    base.clone(),
                    hir::Exp::Temp(operand::Temp::Ret(0)),
                ));
                seq.push(hir::Stm::Move(base.clone(), hir::Exp::Int(vs.len() as i64)));

                for (i, v) in vs.iter().enumerate() {
                    let entry = self.emit_exp(v, vars).into();
                    let offset = hir::Exp::Int((i + 1) as i64 * constants::WORD_SIZE);
                    let address = hir::Exp::Mem(Box::new(hir::Exp::Bin(
                        ir::Bin::Add,
                        Box::new(base.clone()),
                        Box::new(offset),
                    )));
                    seq.push(hir::Stm::Move(address, entry));
                }

                hir::Exp::ESeq(
                    Box::new(hir::Stm::Seq(seq)),
                    Box::new(hir::Exp::Bin(
                        ir::Bin::Add,
                        Box::new(base),
                        Box::new(hir::Exp::Int(constants::WORD_SIZE)),
                    )),
                )
                .into()
            }
            Binary(bin, lhs, rhs, _) => {
                let lhs = self.emit_exp(lhs, vars);
                let rhs = self.emit_exp(rhs, vars);

                if let Some(binop) = match bin {
                    ast::Binary::Mul => Some(ir::Bin::Mul),
                    ast::Binary::Hul => Some(ir::Bin::Hul),
                    ast::Binary::Mod => Some(ir::Bin::Mod),
                    ast::Binary::Div => Some(ir::Bin::Div),
                    ast::Binary::Add => Some(ir::Bin::Add),
                    ast::Binary::Sub => Some(ir::Bin::Sub),
                    _ => None,
                } {
                    return hir::Exp::Bin(binop, Box::new(lhs.into()), Box::new(rhs.into())).into();
                }

                if let Some(relop) = match bin {
                    ast::Binary::Lt => Some(ir::Bin::Lt),
                    ast::Binary::Le => Some(ir::Bin::Le),
                    ast::Binary::Ge => Some(ir::Bin::Ge),
                    ast::Binary::Gt => Some(ir::Bin::Gt),
                    ast::Binary::Ne => Some(ir::Bin::Ne),
                    ast::Binary::Eq => Some(ir::Bin::Eq),
                    _ => None,
                } {
                    return hir::Tree::Cx(Box::new(move |t, f| {
                        let exp = hir::Exp::Bin(relop, Box::new(lhs.into()), Box::new(rhs.into()));
                        hir::Stm::CJump(exp, t, f)
                    }));
                }

                match bin {
                    ast::Binary::And => hir::Tree::Cx(Box::new(move |t, f| {
                        let and = operand::Label::new("AND");
                        hir::Stm::Seq(vec![
                            hir::Con::from(lhs)(and, f),
                            hir::Stm::Label(and),
                            hir::Con::from(rhs)(t, f),
                        ])
                    })),
                    ast::Binary::Or => hir::Tree::Cx(Box::new(move |t, f| {
                        let or = operand::Label::new("OR");
                        hir::Stm::Seq(vec![
                            hir::Con::from(lhs)(t, or),
                            hir::Stm::Label(or),
                            hir::Con::from(rhs)(t, f),
                        ])
                    })),
                    _ => panic!("[INTERNAL ERROR]: missing binary operator in IR emission"),
                }
            }
            Unary(ast::Unary::Neg, exp, _) => hir::Exp::Bin(
                ir::Bin::Sub,
                Box::new(hir::Exp::Int(0)),
                Box::new(self.emit_exp(exp, vars).into()),
            )
            .into(),
            Unary(ast::Unary::Not, exp, _) => hir::Exp::Bin(
                ir::Bin::Xor,
                Box::new(hir::Exp::Int(1)),
                Box::new(self.emit_exp(exp, vars).into()),
            )
            .into(),
            Index(arr, idx, _) => {
                let address = operand::Temp::new("ADDRESS");
                let index = operand::Temp::new("INDEX");
                let length = hir::Exp::Mem(Box::new(hir::Exp::Bin(
                    ir::Bin::Sub,
                    Box::new(hir::Exp::Temp(address)),
                    Box::new(hir::Exp::Int(constants::WORD_SIZE)),
                )));
                let offset = hir::Exp::Bin(
                    ir::Bin::Add,
                    Box::new(hir::Exp::Temp(address)),
                    Box::new(hir::Exp::Bin(
                        ir::Bin::Mul,
                        Box::new(hir::Exp::Temp(index)),
                        Box::new(hir::Exp::Int(constants::WORD_SIZE)),
                    )),
                );

                let lo = operand::Label::new("LOW_BOUND");
                let hi = operand::Label::new("HIGH_BOUND");
                let fail = operand::Label::new("OUT_OF_BOUNDS");
                let safe = operand::Label::new("IN_BOUNDS");
                let oob = hir::Exp::Name(operand::Label::Fix(symbol::intern(
                    constants::XI_OUT_OF_BOUNDS,
                )));

                let lt = hir::Exp::Bin(
                    ir::Bin::Lt,
                    Box::new(hir::Exp::Temp(index)),
                    Box::new(hir::Exp::Int(0)),
                );

                let ge = hir::Exp::Bin(
                    ir::Bin::Ge,
                    Box::new(hir::Exp::Temp(index)),
                    Box::new(length),
                );

                let bounds = hir::Stm::Seq(vec![
                    hir::Stm::Move(hir::Exp::Temp(address), self.emit_exp(&*arr, vars).into()),
                    hir::Stm::Move(hir::Exp::Temp(index), self.emit_exp(&*idx, vars).into()),
                    hir::Stm::CJump(lt, fail, lo),
                    hir::Stm::Label(lo),
                    hir::Stm::CJump(ge, fail, hi),
                    hir::Stm::Label(hi),
                    hir::Stm::Jump(hir::Exp::Name(safe)),
                    hir::Stm::Label(fail),
                    hir::Stm::Call(oob, Vec::with_capacity(0)),
                    hir::Stm::Label(safe),
                ]);

                hir::Exp::ESeq(Box::new(bounds), Box::new(offset)).into()
            }
            Call(call) => {
                let call = self.emit_call(call, vars);
                let temp = hir::Exp::Temp(operand::Temp::Ret(0));
                let save = hir::Exp::Temp(operand::Temp::new("SAVE"));
                let into = hir::Stm::Move(save.clone(), temp);
                hir::Exp::ESeq(Box::new(hir::Stm::Seq(vec![call, into])), Box::new(save)).into()
            }
        }
    }

    fn emit_call(
        &mut self,
        call: &ast::Call,
        vars: &HashMap<symbol::Symbol, operand::Temp>,
    ) -> hir::Stm {
        if symbol::resolve(call.name) == "length" {
            let address = self.emit_exp(&call.arguments[0], vars).into();
            hir::Stm::Move(
                hir::Exp::Mem(Box::new(address)),
                hir::Exp::Temp(operand::Temp::Ret(0)),
            )
        } else {
            hir::Stm::Call(
                hir::Exp::Name(operand::Label::Fix(self.mangle_fun(call.name))),
                call.arguments
                    .iter()
                    .map(|arg| self.emit_exp(arg, vars).into())
                    .collect(),
            )
        }
    }

    fn emit_alloc(length: usize) -> hir::Stm {
        let label = operand::Label::Fix(symbol::intern(constants::XI_ALLOC));
        let alloc = hir::Exp::Name(label);
        let size = hir::Exp::Int((length + 1) as i64 * constants::WORD_SIZE);
        hir::Stm::Call(alloc, vec![size])
    }

    fn emit_dec(
        &mut self,
        dec: &ast::Declaration,
        vars: &mut HashMap<symbol::Symbol, operand::Temp>,
    ) -> hir::Exp {
        let temp = operand::Temp::new("TEMP");
        vars.insert(dec.name, temp);
        hir::Exp::Temp(temp)
    }

    fn emit_stm(
        &mut self,
        stm: &ast::Statement,
        vars: &mut HashMap<symbol::Symbol, operand::Temp>,
    ) -> hir::Stm {
        use ast::Statement::*;
        match stm {
            Assignment(lhs, rhs, _) => {
                let lhs = self.emit_exp(lhs, vars).into();
                let rhs = self.emit_exp(rhs, vars).into();
                hir::Stm::Move(lhs, rhs)
            }
            Call(call) => self.emit_call(call, vars),
            Initialization(decs, ast::Expression::Call(call), _) => {
                let mut seq = vec![self.emit_call(call, vars)];

                for (i, dec) in decs.iter().enumerate() {
                    if let Some(dec) = dec {
                        let var = self.emit_dec(dec, vars);
                        let ret = hir::Exp::Temp(operand::Temp::Ret(i));
                        seq.push(hir::Stm::Move(var, ret));
                    }
                }

                hir::Stm::Seq(seq)
            }
            Initialization(decs, exp, _) => {
                assert!(decs.len() == 1 && decs[0].is_some());
                let dec = decs[0].as_ref().unwrap();
                let var = self.emit_dec(dec, vars);
                let exp = self.emit_exp(exp, vars).into();
                hir::Stm::Move(var, exp)
            }
            Declaration(dec, _) => hir::Stm::Move(self.emit_dec(dec, vars), hir::Exp::Int(0)),
            Return(exps, _) => hir::Stm::Return(
                exps.iter()
                    .map(|exp| self.emit_exp(exp, vars).into())
                    .collect(),
            ),
            Sequence(stms, _) => {
                hir::Stm::Seq(stms.iter().map(|stm| self.emit_stm(stm, vars)).collect())
            }
            If(cond, pass, None, _) => {
                let t = operand::Label::new("TRUE");
                let f = operand::Label::new("FALSE");
                hir::Stm::Seq(vec![
                    hir::Con::from(self.emit_exp(cond, vars))(t, f),
                    hir::Stm::Label(t),
                    self.emit_stm(pass, vars),
                    hir::Stm::Label(f),
                ])
            }
            If(cond, pass, Some(fail), _) => {
                let t = operand::Label::new("TRUE");
                let f = operand::Label::new("FALSE");
                let done = operand::Label::new("DONE");
                hir::Stm::Seq(vec![
                    hir::Con::from(self.emit_exp(cond, vars))(t, f),
                    hir::Stm::Label(t),
                    self.emit_stm(pass, vars),
                    hir::Stm::Jump(hir::Exp::Name(done)),
                    hir::Stm::Label(f),
                    self.emit_stm(fail, vars),
                    hir::Stm::Label(done),
                ])
            }
            While(cond, body, _) => {
                let h = operand::Label::new("WHILE");
                let t = operand::Label::new("TRUE");
                let f = operand::Label::new("FALSE");
                hir::Stm::Seq(vec![
                    hir::Stm::Label(h),
                    hir::Con::from(self.emit_exp(cond, vars))(t, f),
                    self.emit_stm(body, vars),
                    hir::Stm::Jump(hir::Exp::Name(h)),
                    hir::Stm::Label(f),
                ])
            }
        }
    }

    fn mangle_fun(&mut self, fun: symbol::Symbol) -> symbol::Symbol {
        if let Some(mangled) = self.funs.get(&fun) {
            return *mangled;
        }

        let (parameters, returns) = match self.env.get(fun) {
            Some(check::Entry::Function(parameters, returns))
            | Some(check::Entry::Signature(parameters, returns)) => (parameters, returns),
            _ => panic!("[INTERNAL ERROR]: type checking failed"),
        };

        let mut mangled = format!("_I{}_", symbol::resolve(fun).replace('_', "__"),);

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

        for r#type in parameters {
            Self::mangle_type(r#type, &mut mangled);
        }

        let mangled = symbol::intern(mangled);
        self.funs.insert(fun, mangled);
        mangled
    }

    fn mangle_type(typ: &r#type::Expression, mangled: &mut String) {
        match typ {
            r#type::Expression::Any => panic!("[INTERNAL ERROR]: any type in IR"),
            r#type::Expression::Integer => mangled.push('i'),
            r#type::Expression::Boolean => mangled.push('b'),
            r#type::Expression::Array(typ) => {
                mangled.push('a');
                Self::mangle_type(&*typ, mangled);
            }
        }
    }
}
