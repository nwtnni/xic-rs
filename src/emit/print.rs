use std::borrow;
use std::iter;

use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand::Immediate;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::sexp::Serialize;
use crate::data::sexp::Sexp;

impl<T: Serialize> Serialize for ir::Unit<T> {
    fn sexp(&self) -> Sexp {
        iter::once("COMPUNIT".sexp())
            .chain(iter::once(self.name.sexp()))
            .chain(self.functions.values().map(|function| function.sexp()))
            .collect::<Vec<_>>()
            .sexp_move()
    }
}

impl Serialize for hir::Function {
    fn sexp(&self) -> Sexp {
        ["FUNC".sexp(), self.name.sexp(), self.statement.sexp()].sexp_move()
    }
}

impl Serialize for hir::Expression {
    fn sexp(&self) -> Sexp {
        use hir::Expression::*;
        match self {
            Immediate(immediate) => immediate.sexp(),
            Memory(expression) => ["MEM".sexp(), expression.sexp()].sexp_move(),
            Binary(binary, left, right) => [binary.sexp(), left.sexp(), right.sexp()].sexp_move(),
            Call(name, arguments, _) => iter::once("CALL".sexp())
                .chain(iter::once(name.sexp()))
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
                left,
                right,
                r#true,
                r#false,
            } => [
                "CJUMP".sexp(),
                [condition.sexp(), left.sexp(), right.sexp()].sexp_move(),
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
            Return(returns) => iter::once("RETURN".sexp())
                .chain(returns.iter().map(|r#return| r#return.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
            Sequence(statements) => iter::once("SEQ".sexp())
                .chain(statements.iter().map(|statement| statement.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
        }
    }
}

impl<T: Serialize + lir::Target> Serialize for lir::Function<T> {
    fn sexp(&self) -> Sexp {
        [
            "FUNC".sexp(),
            self.name.sexp(),
            iter::once("SEQ".sexp())
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
                let call = iter::once("CALL".sexp())
                    .chain(iter::once(function.sexp()))
                    .chain(arguments.iter().map(|argument| argument.sexp()))
                    .collect::<Vec<_>>()
                    .sexp_move();

                ["EXP".sexp(), call].sexp_move()
            }
            Jump(label) => ["JUMP".sexp(), label.sexp()].sexp_move(),
            CJump {
                condition,
                left,
                right,
                r#true,
                r#false,
            } => {
                let mut sexp = Vec::with_capacity(4);
                sexp.push("CJUMP".sexp());
                sexp.push([condition.sexp(), left.sexp(), right.sexp()].sexp_move());
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
            Return(returns) => iter::once("RETURN".sexp())
                .chain(returns.iter().map(|r#return| r#return.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
        }
    }
}

impl Serialize for lir::Fallthrough {
    fn sexp(&self) -> Sexp {
        Sexp::Atom(borrow::Cow::default())
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
            And => "AND",
            Or => "OR",
        }
        .sexp()
    }
}

impl Serialize for ir::Condition {
    fn sexp(&self) -> Sexp {
        use ir::Condition::*;
        match self {
            Lt => "LT",
            Le => "LEQ",
            Ge => "GEQ",
            Gt => "GT",
            Ne => "NEQ",
            Eq => "EQ",
            Ae => "AEQ",
        }
        .sexp()
    }
}

impl Serialize for Immediate {
    fn sexp(&self) -> Sexp {
        match self {
            Immediate::Integer(integer) => ["CONST".sexp(), integer.sexp()].sexp_move(),
            Immediate::Label(label) => ["NAME".sexp(), label.sexp()].sexp_move(),
        }
    }
}

impl Serialize for Label {
    fn sexp(&self) -> Sexp {
        self.to_string().sexp_move()
    }
}

impl Serialize for Temporary {
    fn sexp(&self) -> Sexp {
        self.to_string().sexp_move()
    }
}
