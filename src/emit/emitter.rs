use std::collections::HashMap;

use crate::check;
use crate::data::ast;
use crate::data::ir;
use crate::data::hir;
use crate::data::typ;
use crate::data::operand;
use crate::util::symbol;

#[derive(Debug)]
pub struct Emitter {
    env: check::Env,
    data: HashMap<symbol::Symbol, operand::Label>,
    funs: HashMap<symbol::Symbol, symbol::Symbol>,
}

const XI_ALLOC: &'static str = "_xi_alloc";
const XI_OUT_OF_BOUNDS: &'static str = "_xi_out_of_bounds";
const WORD_SIZE: usize = 8;

impl Emitter {
    pub fn emit_program(mut self, ast: &ast::Program) -> ir::Unit<hir::Fun> {
        let mut funs = HashMap::with_capacity(ast.funs.len());
        for fun in &ast.funs {
            let ir = self.emit_fun(fun);
            let id = self.mangle_fun(fun.name);
            funs.insert(id, ir);
        }
        ir::Unit {
            name: symbol::intern("program"),
            funs: funs,
            data: self.data,
        }
    }

    fn emit_fun(&mut self, fun: &ast::Fun) -> hir::Fun {
        let mut vars = HashMap::default();
        hir::Fun {
            name: self.mangle_fun(fun.name),
            body: self.emit_stm(&fun.body, &mut vars),
            vars: vars,
        }
    }

    fn emit_exp(&mut self, exp: &ast::Exp, vars: &HashMap<symbol::Symbol, operand::Temp>) -> hir::Tree {
        use ast::Exp::*;
        match exp {
        | Bool(false, _) => hir::Exp::Int(0).into(),
        | Bool(true, _) => hir::Exp::Int(1).into(),
        | Int(i, _) => hir::Exp::Int(*i).into(),
        | Chr(c, _) => hir::Exp::Int(*c as i64).into(),
        | Str(s, _) => {
            let symbol = symbol::intern(s);
            let label = *self.data
                .entry(symbol)
                .or_insert_with(|| operand::Label::new("STR"));
            hir::Exp::Name(label).into()
        }
        | Var(v, _) => hir::Exp::Temp(vars[v]).into(),
        | Arr(vs, _) => {

            let alloc = Self::emit_alloc(vs.len());
            let base = hir::Exp::Temp(operand::Temp::new("ARR"));

            let mut seq = Vec::with_capacity(vs.len() + 2);
            seq.push(hir::Stm::Move(base.clone(), alloc));
            seq.push(hir::Stm::Move(base.clone(), hir::Exp::Int(vs.len() as i64)));

            for (i, v) in vs.iter().enumerate() {
                let entry = self.emit_exp(v, vars).into();
                let offset = hir::Exp::Int(((i + 1) * WORD_SIZE) as i64);
                let address = hir::Exp::Mem(Box::new(
                    hir::Exp::Bin(
                        ir::Bin::Add,
                        Box::new(base.clone()),
                        Box::new(offset)
                    ),
                ));
                seq.push(hir::Stm::Move(address, entry));
            }

            hir::Exp::ESeq(
                Box::new(hir::Stm::Seq(seq)),
                Box::new(base)
            ).into()
        }
        | Bin(bin, lhs, rhs, _) => {
            let lhs = self.emit_exp(lhs, vars);
            let rhs = self.emit_exp(rhs, vars);

            if let Some(binop) = match bin {
            | ast::Bin::Mul => Some(ir::Bin::Mul),
            | ast::Bin::Hul => Some(ir::Bin::Hul),
            | ast::Bin::Mod => Some(ir::Bin::Mod),
            | ast::Bin::Div => Some(ir::Bin::Div),
            | ast::Bin::Add => Some(ir::Bin::Add),
            | ast::Bin::Sub => Some(ir::Bin::Sub),
            | _ => None,
            } {
                return hir::Exp::Bin(
                    binop,
                    Box::new(lhs.into()),
                    Box::new(rhs.into()),
                ).into()
            }

            if let Some(relop) = match bin {
            | ast::Bin::Lt => Some(ir::Rel::Lt),
            | ast::Bin::Le => Some(ir::Rel::Le),
            | ast::Bin::Ge => Some(ir::Rel::Ge),
            | ast::Bin::Gt => Some(ir::Rel::Gt),
            | ast::Bin::Ne => Some(ir::Rel::Ne),
            | ast::Bin::Eq => Some(ir::Rel::Eq),
            | _ => None,
            } {
                return hir::Tree::Cx(Box::new(move |t, f| {
                    hir::Stm::CJump(relop, lhs.into(), rhs.into(), t, f)
                }))
            }

            match bin {
            | ast::Bin::And => {
                hir::Tree::Cx(Box::new(move |t, f| {
                    let and = operand::Label::new("AND");
                    hir::Stm::Seq(vec![
                        hir::Con::from(lhs)(and, f),
                        hir::Stm::Label(and),
                        hir::Con::from(rhs)(t, f),
                    ])
                }))
            }
            | ast::Bin::Or => {
                hir::Tree::Cx(Box::new(move |t, f| {
                    let or = operand::Label::new("OR");
                    hir::Stm::Seq(vec![
                        hir::Con::from(lhs)(t, or),
                        hir::Stm::Label(or),
                        hir::Con::from(rhs)(t, f),
                    ])
                }))
            }
            | _ => panic!("[INTERNAL ERROR]: missing binary operator in IR emission"),
            }
        }
        | Uno(ast::Uno::Neg, exp, _) => {
            hir::Exp::Bin(
                ir::Bin::Sub, 
                Box::new(hir::Exp::Int(0)),
                Box::new(self.emit_exp(exp, vars).into()),
            ).into()
        }
        | Uno(ast::Uno::Not, exp, _) => {
            hir::Exp::Bin(
                ir::Bin::Xor,
                Box::new(hir::Exp::Int(1)),
                Box::new(self.emit_exp(exp, vars).into()),
            ).into()
        }
        | Idx(arr, idx, _) => {
            let address = hir::Exp::from(self.emit_exp(&*arr, vars));
            let memory = hir::Exp::Mem(Box::new(address.clone()));
            let index = hir::Exp::Temp(operand::Temp::new("INDEX"));
            let offset = hir::Exp::Bin(
                ir::Bin::Add,
                Box::new(address),
                Box::new(hir::Exp::Bin(
                    ir::Bin::Mul,
                    Box::new(hir::Exp::Int(WORD_SIZE as i64)),
                    Box::new(hir::Exp::Bin(
                        ir::Bin::Add,
                        Box::new(hir::Exp::Int(1)),
                        Box::new(index.clone()),
                    )),
                )),
            );

            let lo = operand::Label::new("LOW_BOUND");
            let hi = operand::Label::new("HIGH_BOUND");
            let fail = operand::Label::Fix(symbol::intern(XI_OUT_OF_BOUNDS));

            let bounds = hir::Stm::Seq(vec![
                hir::Stm::Move(index.clone(), self.emit_exp(&*idx, vars).into()),
                hir::Stm::CJump(ir::Rel::Lt, index.clone(), hir::Exp::Int(0), fail, lo),
                hir::Stm::Label(lo),
                hir::Stm::CJump(ir::Rel::Ge, index.clone(), memory.clone(), fail, hi),
                hir::Stm::Label(hi),
            ]);

            hir::Exp::ESeq(Box::new(bounds), Box::new(offset)).into()
        }
        | Call(ast::Call { name, args, ..}) if *name == symbol::intern("length") => {
            let address = self.emit_exp(&args[0], vars).into();
            hir::Exp::Mem(Box::new(address)).into()
        }
        | Call(call) => self.emit_call(call, vars).into(),
        }
    }

    fn emit_call(&mut self, call: &ast::Call, vars: &HashMap<symbol::Symbol, operand::Temp>) -> hir::Exp {
        hir::Exp::Call(
            Box::new(hir::Exp::Name(operand::Label::Fix(self.mangle_fun(call.name)))),
            call.args.iter()
                .map(|arg| self.emit_exp(arg, vars).into())
                .collect(),
        )
    }

    fn emit_alloc(length: usize) -> hir::Exp {
        let label = operand::Label::Fix(symbol::intern(XI_ALLOC));
        let alloc = hir::Exp::Name(label);
        let size = hir::Exp::Int(((length + 1) * WORD_SIZE) as i64);
        hir::Exp::Call(Box::new(alloc), vec![size])
    }

    fn emit_dec(&mut self, dec: &ast::Dec, vars: &mut HashMap<symbol::Symbol, operand::Temp>) -> hir::Exp {
        let temp = operand::Temp::new("TEMP");
        vars.insert(dec.name, temp);
        hir::Exp::Temp(temp)
    }

    fn emit_stm(&mut self, stm: &ast::Stm, vars: &mut HashMap<symbol::Symbol, operand::Temp>) -> hir::Stm {
        use ast::Stm::*;
        match stm {
        | Ass(lhs, rhs, _) => {
            let lhs = self.emit_exp(lhs, vars).into();
            let rhs = self.emit_exp(rhs, vars).into();
            hir::Stm::Move(lhs, rhs).into()
        }
        | Call(call) => hir::Stm::Exp(self.emit_call(call, vars)).into(),
        | Init(decs, exp, _) if decs.len() == 1 && decs[0].is_some() => {
            let dec = decs[0].as_ref().unwrap();
            let exp = self.emit_exp(exp, vars).into();
            let var = self.emit_dec(dec, vars);
            hir::Stm::Move(var, exp)
        }
        | Init(decs, exp, _) => {
            let mut seq = Vec::new();

            seq.push(hir::Stm::Exp(self.emit_exp(exp, vars).into()));

            for (i, dec) in decs.iter().enumerate() {
                if let Some(dec) = dec {
                    let var = self.emit_dec(dec, vars);
                    let ret = hir::Exp::Temp(operand::Temp::Ret(i));
                    seq.push(hir::Stm::Move(var, ret));
                }
            }

            hir::Stm::Seq(seq)
        }
        | Dec(dec, _) => hir::Stm::Move(self.emit_dec(dec, vars), hir::Exp::Int(0)),
        | Ret(exps, _) => {
            hir::Stm::Return(
                exps.iter()
                    .map(|exp| self.emit_exp(exp, vars).into())
                    .collect()
            )
        }
        | Seq(stms, _) => {
            hir::Stm::Seq(
                stms.iter()
                    .map(|stm| self.emit_stm(stm, vars))
                    .collect()
            )
        }
        | If(cond, pass, None, _) => {
            let t = operand::Label::new("TRUE");
            let f = operand::Label::new("FALSE");
            hir::Stm::Seq(vec![
                hir::Con::from(self.emit_exp(cond, vars))(t, f),
                hir::Stm::Label(t),
                self.emit_stm(pass, vars),
                hir::Stm::Label(f),
            ])
        }
        | If(cond, pass, Some(fail), _) => {
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
        | While(cond, body, _) => {
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
            return *mangled
        }

        let (is, os) = match self.env.get(fun) {
        | Some(check::Entry::Fun(is, os))
        | Some(check::Entry::Sig(is, os)) => (is, os),
        | _ => panic!("[INTERNAL ERROR]: type checking failed"),
        };

        let mut mangled = format!(
            "_I{}_",
            symbol::resolve(fun).replace("_", "__"),
        );

        match os {
        | typ::Typ::Unit => mangled.push('p'),
        | typ::Typ::Exp(typ) => {
            Self::mangle_typ(typ, &mut mangled);
        }
        | typ::Typ::Tup(typs) => {
            mangled.push('t');
            mangled.push_str(&typs.len().to_string());
            for typ in typs { Self::mangle_typ(typ, &mut mangled); }
        }
        }

        for typ in is { Self::mangle_typ(typ, &mut mangled); }

        let mangled = symbol::intern(mangled);
        self.funs.insert(fun, mangled);
        mangled
    }

    fn mangle_typ(typ: &typ::Exp, mangled: &mut String) {
        match typ {
        | typ::Exp::Any => panic!("[INTERNAL ERROR]: any type in IR"),
        | typ::Exp::Int => mangled.push('i'),
        | typ::Exp::Bool => mangled.push('b'),
        | typ::Exp::Arr(typ) => {
            mangled.push('a');
            Self::mangle_typ(&*typ, mangled);
        }
        }
    }
}
