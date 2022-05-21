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
        match self {
            ast::Global::Declaration(declaration) => declaration.sexp(),
            ast::Global::Initialization(initialization) => initialization.sexp(),
        }
    }
}

impl Serialize for ast::Initialization {
    fn sexp(&self) -> Sexp {
        let mut declarations = self
            .declarations
            .iter()
            .map(|declaration| {
                declaration
                    .as_ref()
                    .map(Serialize::sexp)
                    .unwrap_or_else(|| "_".sexp())
            })
            .collect::<Vec<_>>();

        let declarations = if declarations.len() == 1 {
            declarations.remove(0)
        } else {
            Sexp::List(declarations)
        };

        ["=".sexp(), declarations.sexp(), self.expression.sexp()].sexp_move()
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
            Class(class) => class.sexp(),
            Array(r#type, None, _) => ["[]".sexp(), r#type.sexp()].sexp_move(),
            Array(r#type, Some(length), _) => {
                ["[]".sexp(), r#type.sexp(), length.sexp()].sexp_move()
            }
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
            Character(char, _) => match token::unescape_char(*char) {
                Some(string) => format!("\'{}\'", string).sexp_move(),
                None => format!("\'{}\'", char).sexp_move(),
            },
            String(string, _) => format!("\"{}\"", token::unescape_str(string)).sexp_move(),
            Integer(integer, _) if *integer < 0 => {
                ["-".sexp(), (-(*integer as i128)).to_string().sexp_move()].sexp_move()
            }
            Integer(integer, _) => integer.to_string().sexp_move(),
            Null(_) => "null".sexp(),
            This(_) => "this".sexp(),
            Variable(variable) => variable.sexp(),
            Array(expressions, _) => expressions.sexp(),
            Binary(binary, left, right, _) => {
                [binary.get().sexp(), left.sexp(), right.sexp()].sexp_move()
            }
            Unary(unary, expression, _) => [unary.sexp(), expression.sexp()].sexp_move(),
            Index(array, index, _) => ["[]".sexp(), array.sexp(), index.sexp()].sexp_move(),
            Length(array, _) => ["length".sexp(), array.sexp()].sexp_move(),
            Dot(expression, symbol, _) => {
                [".".sexp(), expression.sexp(), symbol.sexp()].sexp_move()
            }
            New(symbol, _) => ["new".sexp(), symbol.sexp()].sexp_move(),
            Call(call) => call.sexp(),
        }
    }
}

impl Serialize for ast::Declaration {
    fn sexp(&self) -> Sexp {
        match self {
            ast::Declaration::Multiple(multiple) => multiple.sexp(),
            ast::Declaration::Single(single) => single.sexp(),
        }
    }
}

impl Serialize for ast::MultipleDeclaration {
    fn sexp(&self) -> Sexp {
        [self.names.sexp(), self.r#type.sexp()].sexp_move()
    }
}

impl Serialize for ast::SingleDeclaration {
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
        args.insert(0, self.function.sexp());
        args.sexp_move()
    }
}

impl Serialize for ast::Statement {
    fn sexp(&self) -> Sexp {
        use ast::Statement::*;
        match self {
            Assignment(left, right, _) => ["=".sexp(), left.sexp(), right.sexp()].sexp_move(),
            Call(call) => call.sexp(),
            Initialization(initialization) => initialization.sexp(),
            Declaration(declaration, _) => declaration.sexp(),
            Return(expressions, _) => std::iter::once("return".sexp())
                .chain(expressions.iter().map(Serialize::sexp))
                .collect::<Vec<_>>()
                .tap(Sexp::List),
            Sequence(statements, _) => statements.sexp(),
            If(condition, r#if, Some(r#else), _) => {
                ["if".sexp(), condition.sexp(), r#if.sexp(), r#else.sexp()].sexp_move()
            }
            If(condition, r#if, None, _) => {
                ["if".sexp(), condition.sexp(), r#if.sexp()].sexp_move()
            }
            While(ast::Do::Yes, condition, body, _) => {
                ["do".sexp(), body.sexp(), "while".sexp(), condition.sexp()].sexp_move()
            }
            While(ast::Do::No, condition, body, _) => {
                ["while".sexp(), condition.sexp(), body.sexp()].sexp_move()
            }
            Break(_) => "break".sexp(),
        }
    }
}

impl Serialize for ast::Identifier {
    fn sexp(&self) -> Sexp {
        self.symbol.sexp()
    }
}
