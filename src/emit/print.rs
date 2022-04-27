use std::borrow;

use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::sexp::{Serialize, Sexp};

impl<T: Serialize> Serialize for ir::Unit<T> {
    fn sexp(&self) -> Sexp {
        std::iter::once("COMPUNIT".sexp())
            .chain(std::iter::once(self.name.sexp()))
            .chain(self.functions.values().map(|function| function.sexp()))
            .collect::<Vec<_>>()
            .sexp_move()
    }
}

impl Serialize for hir::Function {
    fn sexp(&self) -> Sexp {
        ["FUNC".sexp(), self.name.sexp(), self.statements.sexp()].sexp_move()
    }
}

impl Serialize for hir::Expression {
    fn sexp(&self) -> Sexp {
        use hir::Expression::*;
        match self {
            Immediate(immediate) => immediate.sexp(),
            Memory(expression) => ["MEM".sexp(), expression.sexp()].sexp_move(),
            Binary(binary, left, right) => [binary.sexp(), left.sexp(), right.sexp()].sexp_move(),
            Call(name, arguments, _) => std::iter::once("CALL".sexp())
                .chain(std::iter::once(name.sexp()))
                .chain(arguments.iter().map(|argument| argument.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
            Temporary(temporary) => ["TEMP".sexp(), temporary.sexp()].sexp_move(),
            Sequence(sequence, expression) => {
                ["ESEQ".sexp(), sequence.sexp(), expression.sexp()].sexp_move()
            }
        }
    }
}

impl Serialize for hir::Statement {
    fn sexp(&self) -> Sexp {
        use hir::Statement::*;
        match self {
            Jump(label) => ["JUMP".sexp(), label.sexp()].sexp_move(),
            CJump {
                condition,
                r#true,
                r#false,
            } => [
                "CJUMP".sexp(),
                condition.sexp(),
                r#true.sexp(),
                r#false.sexp(),
            ]
            .sexp_move(),
            Label(label) => ["LABEL".sexp(), label.sexp()].sexp_move(),
            Expression(expression) => ["EXP".sexp(), expression.sexp()].sexp_move(),
            Move {
                destination,
                source,
            } => ["MOVE".sexp(), destination.sexp(), source.sexp()].sexp_move(),
            Return => ["RETURN".sexp()].sexp_move(),
            Sequence(statements) => std::iter::once("SEQ".sexp())
                .chain(statements.iter().map(|statement| statement.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
        }
    }
}

impl<T: Serialize> Serialize for lir::Function<T> {
    fn sexp(&self) -> Sexp {
        [
            "FUNC".sexp(),
            self.name.sexp(),
            std::iter::once("SEQ".sexp())
                .chain(self.statements.iter().map(|statement| statement.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
        ]
        .sexp_move()
    }
}

impl Serialize for lir::Expression {
    fn sexp(&self) -> Sexp {
        use lir::Expression::*;
        match self {
            Immediate(immediate) => immediate.sexp(),
            Memory(expression) => ["MEM".sexp(), expression.sexp()].sexp_move(),
            Binary(binary, left, right) => [binary.sexp(), left.sexp(), right.sexp()].sexp_move(),
            Temporary(temporary) => ["TEMP".sexp(), temporary.sexp()].sexp_move(),
        }
    }
}

impl<T: Serialize> Serialize for lir::Statement<T> {
    fn sexp(&self) -> Sexp {
        use lir::Statement::*;
        match self {
            Call(function, arguments, _) => {
                let call = std::iter::once("CALL".sexp())
                    .chain(std::iter::once(function.sexp()))
                    .chain(arguments.iter().map(|argument| argument.sexp()))
                    .collect::<Vec<_>>()
                    .sexp_move();

                ["EXP".sexp(), call].sexp_move()
            }
            Jump(label) => ["JUMP".sexp(), label.sexp()].sexp_move(),
            CJump {
                condition,
                r#true,
                r#false,
            } => {
                let mut sexp = Vec::with_capacity(4);
                sexp.push("CJUMP".sexp());
                sexp.push(condition.sexp());
                sexp.push(r#true.sexp());

                match r#false.sexp() {
                    Sexp::Atom(atom) if atom.is_empty() => (),
                    r#false => sexp.push(r#false),
                }

                sexp.sexp_move()
            }
            .sexp_move(),
            Label(label) => ["LABEL".sexp(), label.sexp()].sexp_move(),
            Move {
                destination,
                source,
            } => ["MOVE".sexp(), destination.sexp(), source.sexp()].sexp_move(),
            Return => ["RETURN".sexp()].sexp_move(),
        }
    }
}

impl Serialize for lir::Fallthrough {
    fn sexp(&self) -> Sexp {
        Sexp::Atom(borrow::Cow::default())
    }
}

impl Serialize for lir::Label {
    fn sexp(&self) -> Sexp {
        self.0.sexp()
    }
}

impl Serialize for ir::Binary {
    fn sexp(&self) -> Sexp {
        use ir::Binary::*;
        match self {
            Add => "ADD",
            Sub => "SUB",
            Mul => "MUL",
            Hul => "HMUL",
            Div => "DIV",
            Mod => "MOD",
            Xor => "XOR",
            Ls => "LSHIFT",
            Rs => "RSHIFT",
            ARs => "ARSHIFT",
            And => "AND",
            Or => "OR",
            Lt => "LT",
            Le => "LEQ",
            Ge => "GEQ",
            Gt => "GT",
            Ne => "NEQ",
            Eq => "EQ",
        }
        .sexp()
    }
}

impl Serialize for operand::Immediate {
    fn sexp(&self) -> Sexp {
        match self {
            operand::Immediate::Integer(integer) => ["CONST".sexp(), integer.sexp()].sexp_move(),
            operand::Immediate::Label(label) => ["NAME".sexp(), label.sexp()].sexp_move(),
        }
    }
}

impl Serialize for operand::Label {
    fn sexp(&self) -> Sexp {
        self.to_string().sexp_move()
    }
}

impl Serialize for operand::Temporary {
    fn sexp(&self) -> Sexp {
        self.to_string().sexp_move()
    }
}
