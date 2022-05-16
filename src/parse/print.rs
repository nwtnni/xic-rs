use crate::data::ast;
use crate::data::sexp::Serialize;
use crate::data::sexp::Sexp;
use crate::data::token;
use crate::util::Tap;

impl Serialize for ast::Interface {
    fn sexp(&self) -> Sexp {
        [self.uses.sexp(), self.items.sexp()].sexp_move()
    }
}

impl Serialize for ast::Program {
    fn sexp(&self) -> Sexp {
        [self.uses.sexp(), self.items.sexp()].sexp_move()
    }
}

impl Serialize for ast::Use {
    fn sexp(&self) -> Sexp {
        ["use".sexp(), self.name.sexp()].sexp_move()
    }
}

impl Serialize for ast::ItemSignature {
    fn sexp(&self) -> Sexp {
        match self {
            ast::ItemSignature::Class(class) => class.sexp(),
            ast::ItemSignature::Function(function) => function.sexp(),
        }
    }
}

impl Serialize for ast::Item {
    fn sexp(&self) -> Sexp {
        match self {
            ast::Item::Global(global) => global.sexp(),
            ast::Item::Class(class) => class.sexp(),
            ast::Item::Function(function) => function.sexp(),
        }
    }
}

impl Serialize for ast::Global {
    fn sexp(&self) -> Sexp {
        match &self.value {
            None => self.declaration.sexp(),
            Some(expression) => {
                ["=".sexp(), self.declaration.sexp(), expression.sexp()].sexp_move()
            }
        }
    }
}

impl Serialize for ast::ClassSignature {
    fn sexp(&self) -> Sexp {
        [
            self.name.sexp(),
            match &self.extends {
                None => ["extends".sexp()].sexp_move(),
                Some(extends) => ["extends".sexp(), extends.sexp()].sexp_move(),
            },
            self.methods.sexp(),
        ]
        .sexp_move()
    }
}

impl Serialize for ast::Class {
    fn sexp(&self) -> Sexp {
        [
            self.name.sexp(),
            if let Some(extends) = &self.extends {
                ["extends".sexp(), extends.sexp()].sexp_move()
            } else {
                Vec::<&'static str>::new().sexp_move()
            },
            self.items.sexp(),
        ]
        .sexp_move()
    }
}

impl Serialize for ast::ClassItem {
    fn sexp(&self) -> Sexp {
        match self {
            ast::ClassItem::Field(field) => field.sexp(),
            ast::ClassItem::Method(method) => method.sexp(),
        }
    }
}

impl Serialize for ast::FunctionSignature {
    fn sexp(&self) -> Sexp {
        [
            self.name.sexp(),
            self.parameters.sexp(),
            self.returns.sexp(),
        ]
        .sexp_move()
    }
}

impl Serialize for ast::Function {
    fn sexp(&self) -> Sexp {
        [
            self.name.sexp(),
            self.parameters.sexp(),
            self.returns.sexp(),
            self.statements.sexp(),
        ]
        .sexp_move()
    }
}

impl Serialize for ast::Type {
    fn sexp(&self) -> Sexp {
        use ast::Type::*;
        match self {
            Bool(_) => "bool".sexp(),
            Int(_) => "int".sexp(),
            Array(typ, None, _) => ["[]".sexp(), typ.sexp()].sexp_move(),
            Array(typ, Some(exp), _) => ["[]".sexp(), typ.sexp(), exp.sexp()].sexp_move(),
        }
    }
}

impl Serialize for ast::Binary {
    fn sexp(&self) -> Sexp {
        use ast::Binary::*;
        match self {
            Mul => "*",
            Hul => "*>>",
            Div => "/",
            Mod => "%",
            Add | Cat => "+",
            Sub => "-",
            Lt => "<",
            Le => "<=",
            Ge => ">=",
            Gt => ">",
            Eq => "==",
            Ne => "!=",
            And => "&",
            Or => "|",
        }
        .sexp()
    }
}

impl Serialize for ast::Unary {
    fn sexp(&self) -> Sexp {
        use ast::Unary::*;
        match self {
            Neg => "-",
            Not => "!",
        }
        .sexp()
    }
}

impl Serialize for ast::Expression {
    fn sexp(&self) -> Sexp {
        use ast::Expression::*;
        match self {
            Boolean(false, _) => "false".sexp(),
            Boolean(true, _) => "true".sexp(),
            Character(c, _) => match token::unescape_char(*c) {
                Some(s) => format!("\'{}\'", s).sexp_move(),
                None => format!("\'{}\'", c).sexp_move(),
            },
            String(s, _) => format!("\"{}\"", token::unescape_str(s)).sexp_move(),
            Integer(i, _) if *i < 0 => {
                ["-".sexp(), (-(*i as i128)).to_string().sexp_move()].sexp_move()
            }
            Integer(i, _) => i.to_string().sexp_move(),
            Variable(v, _) => v.sexp(),
            Array(exps, _) => exps.sexp(),
            Binary(bin, lhs, rhs, _) => [bin.get().sexp(), lhs.sexp(), rhs.sexp()].sexp_move(),
            Unary(uno, exp, _) => [uno.sexp(), exp.sexp()].sexp_move(),
            Index(arr, idx, _) => ["[]".sexp(), arr.sexp(), idx.sexp()].sexp_move(),
            Call(call) => call.sexp(),
        }
    }
}

impl Serialize for ast::Declaration {
    fn sexp(&self) -> Sexp {
        [self.name.sexp(), self.r#type.sexp()].sexp_move()
    }
}

impl Serialize for ast::Call {
    fn sexp(&self) -> Sexp {
        let mut args = self
            .arguments
            .iter()
            .map(Serialize::sexp)
            .collect::<Vec<_>>();
        args.insert(0, self.name.sexp());
        args.sexp_move()
    }
}

impl Serialize for ast::Statement {
    fn sexp(&self) -> Sexp {
        use ast::Statement::*;
        match self {
            Assignment(lhs, rhs, _) => ["=".sexp(), lhs.sexp(), rhs.sexp()].sexp_move(),
            Call(call) => call.sexp(),
            Initialization(decs, call, _) => {
                let mut decs = decs
                    .iter()
                    .map(|dec| {
                        dec.as_ref()
                            .map(Serialize::sexp)
                            .unwrap_or_else(|| "_".sexp())
                    })
                    .collect::<Vec<_>>();
                let decs = if decs.len() == 1 {
                    decs.remove(0)
                } else {
                    Sexp::List(decs)
                };
                ["=".sexp(), decs.sexp(), call.sexp()].sexp_move()
            }
            Declaration(dec, _) => dec.sexp(),
            Return(exps, _) => std::iter::once("return".sexp())
                .chain(exps.iter().map(Serialize::sexp))
                .collect::<Vec<_>>()
                .tap(Sexp::List),
            Sequence(stms, _) => stms.sexp(),
            If(cond, pass, Some(fail), _) => {
                ["if".sexp(), cond.sexp(), pass.sexp(), fail.sexp()].sexp_move()
            }
            If(cond, pass, None, _) => ["if".sexp(), cond.sexp(), pass.sexp()].sexp_move(),
            While(ast::Do::Yes, cond, body, _) => {
                ["do".sexp(), body.sexp(), "while".sexp(), cond.sexp()].sexp_move()
            }
            While(ast::Do::No, cond, body, _) => {
                ["while".sexp(), cond.sexp(), body.sexp()].sexp_move()
            }
        }
    }
}
