use crate::data::hir;
use crate::data::ir;
use crate::data::lir;
use crate::data::operand;
use crate::data::sexp::{Serialize, Sexp};
use crate::data::symbol;

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
            Integer(integer) => ["CONST".sexp(), integer.sexp()].sexp_move(),
            Memory(expression) => ["MEM".sexp(), expression.sexp()].sexp_move(),
            Binary(binary, left, right) => [binary.sexp(), left.sexp(), right.sexp()].sexp_move(),
            Call(name, arguments) => std::iter::once("CALL".sexp())
                .chain(std::iter::once(name.sexp()))
                .chain(arguments.iter().map(|argument| argument.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
            Label(label) => ["NAME".sexp(), label.sexp()].sexp_move(),
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
            Jump(expression) => ["JUMP".sexp(), expression.sexp()].sexp_move(),
            CJump(condition, r#true, r#false) => [
                "CJUMP".sexp(),
                condition.sexp(),
                r#true.sexp(),
                r#false.sexp(),
            ]
            .sexp_move(),
            Label(label) => ["LABEL".sexp(), label.sexp()].sexp_move(),
            Expression(expression) => ["EXP".sexp(), expression.sexp()].sexp_move(),
            Move(into, from) => ["MOVE".sexp(), into.sexp(), from.sexp()].sexp_move(),
            Return(expressions) => std::iter::once("RETURN".sexp())
                .chain(expressions.iter().map(|expression| expression.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
            Sequence(statements) => std::iter::once("SEQ".sexp())
                .chain(statements.iter().map(|statement| statement.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
        }
    }
}

impl Serialize for lir::Function {
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
            Integer(integer) => ["CONST".sexp(), integer.sexp()].sexp_move(),
            Memory(expression) => ["MEM".sexp(), expression.sexp()].sexp_move(),
            Binary(binary, left, right) => [binary.sexp(), left.sexp(), right.sexp()].sexp_move(),
            Label(label) => ["NAME".sexp(), label.sexp()].sexp_move(),
            Temporary(temporary) => ["TEMP".sexp(), temporary.sexp()].sexp_move(),
        }
    }
}

impl Serialize for lir::Statement {
    fn sexp(&self) -> Sexp {
        use lir::Statement::*;
        match self {
            Call(function, arguments) => {
                let call = std::iter::once("CALL".sexp())
                    .chain(std::iter::once(function.sexp()))
                    .chain(arguments.iter().map(|argument| argument.sexp()))
                    .collect::<Vec<_>>()
                    .sexp_move();

                ["EXP".sexp(), call].sexp_move()
            }
            Jump(expression) => ["JUMP".sexp(), expression.sexp()].sexp_move(),
            CJump(condition, r#true, r#false) => [
                "CJUMP".sexp(),
                condition.sexp(),
                r#true.sexp(),
                r#false.sexp(),
            ]
            .sexp_move(),
            Label(label) => ["LABEL".sexp(), label.sexp()].sexp_move(),
            Move(into, from) => ["MOVE".sexp(), into.sexp(), from.sexp()].sexp_move(),
            Return(expressions) => std::iter::once("RETURN".sexp())
                .chain(expressions.iter().map(|expression| expression.sexp()))
                .collect::<Vec<_>>()
                .sexp_move(),
        }
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

impl Serialize for operand::Label {
    fn sexp(&self) -> Sexp {
        use operand::Label::*;
        match self {
            Fixed(symbol) => symbol.sexp(),
            Fresh(symbol, index) => format!("{}{}", symbol::resolve(*symbol), index).sexp_move(),
        }
    }
}

impl Serialize for operand::Temporary {
    fn sexp(&self) -> Sexp {
        use operand::Temporary::*;
        match self {
            Argument(index) => format!("_ARG{}", index).sexp_move(),
            Return(index) => format!("_RET{}", index).sexp_move(),
            Fresh(symbol, index) => format!("{}{}", symbol::resolve(*symbol), index).sexp_move(),
            Register(_) => panic!("[INTERNAL ERROR]: shouldn't be any registers in IR"),
        }
    }
}
