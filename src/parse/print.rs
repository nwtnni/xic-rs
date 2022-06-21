use crate::data::ast;
use crate::data::sexp::Serialize;
use crate::data::sexp::Sexp;
use crate::data::token;
use crate::util::Tap;

impl<T> Serialize for ast::Interface<T> {
    fn sexp(&self) -> Sexp {
        [self.uses.sexp(), self.items.sexp()].sexp_move()
    }
}

impl<T> Serialize for ast::Program<T> {
    fn sexp(&self) -> Sexp {
        [self.uses.sexp(), self.items.sexp()].sexp_move()
    }
}

impl Serialize for ast::Use {
    fn sexp(&self) -> Sexp {
        ["use".sexp(), self.name.sexp()].sexp_move()
    }
}

impl<T> Serialize for ast::ItemSignature<T> {
    fn sexp(&self) -> Sexp {
        match self {
            ast::ItemSignature::Class(class) => class.sexp(),
            ast::ItemSignature::ClassTemplate(class) => class.sexp(),
            ast::ItemSignature::Function(function) => function.sexp(),
            ast::ItemSignature::FunctionTemplate(function) => function.sexp(),
        }
    }
}

impl<T> Serialize for ast::Item<T> {
    fn sexp(&self) -> Sexp {
        match self {
            ast::Item::Global(global) => global.sexp(),
            ast::Item::Class(class) => class.sexp(),
            ast::Item::ClassTemplate(class) => class.sexp(),
            ast::Item::Function(function) => function.sexp(),
            ast::Item::FunctionTemplate(function) => function.sexp(),
        }
    }
}

impl<T> Serialize for ast::Global<T> {
    fn sexp(&self) -> Sexp {
        match self {
            ast::Global::Declaration(declaration) => declaration.sexp(),
            ast::Global::Initialization(initialization) => initialization.sexp(),
        }
    }
}

impl<T> Serialize for ast::Initialization<T> {
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

impl Serialize for ast::ClassTemplate {
    fn sexp(&self) -> Sexp {
        [
            match self.r#final {
                true => ["final".sexp(), self.name.sexp()].sexp_move(),
                false => self.name.sexp(),
            },
            self.generics.sexp(),
            self.items.sexp(),
        ]
        .sexp_move()
    }
}

impl<T> Serialize for ast::ClassSignature<T> {
    fn sexp(&self) -> Sexp {
        [
            match self.r#final {
                true => ["final".sexp(), self.name.sexp()].sexp_move(),
                false => self.name.sexp(),
            },
            match &self.extends {
                None => ["extends".sexp()].sexp_move(),
                Some(extends) => ["extends".sexp(), extends.sexp()].sexp_move(),
            },
            self.methods.sexp(),
        ]
        .sexp_move()
    }
}

impl<T> Serialize for ast::Class<T> {
    fn sexp(&self) -> Sexp {
        [
            match self.r#final {
                true => ["final".sexp(), self.name.sexp()].sexp_move(),
                false => self.name.sexp(),
            },
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

impl<T> Serialize for ast::ClassItem<T> {
    fn sexp(&self) -> Sexp {
        match self {
            ast::ClassItem::Field(field) => field.sexp(),
            ast::ClassItem::Method(method) => method.sexp(),
        }
    }
}

impl Serialize for ast::FunctionTemplate {
    fn sexp(&self) -> Sexp {
        [
            self.name.sexp(),
            self.generics.sexp(),
            self.parameters.sexp(),
            self.returns.sexp(),
            self.statements.sexp(),
        ]
        .sexp_move()
    }
}

impl<T> Serialize for ast::FunctionSignature<T> {
    fn sexp(&self) -> Sexp {
        [
            self.name.sexp(),
            self.parameters.sexp(),
            self.returns.sexp(),
        ]
        .sexp_move()
    }
}

impl<T> Serialize for ast::Function<T> {
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

impl<T> Serialize for ast::Type<T> {
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

impl<T> Serialize for ast::Expression<T> {
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
            Null(_, _) => "null".sexp(),
            This(_, _) => "this".sexp(),
            Super(_, _) => "super".sexp(),
            Variable(variable, _) => variable.sexp(),
            Array(expressions, _, _) => expressions.sexp(),
            Binary(binary, left, right, _, _) => {
                [binary.get().sexp(), left.sexp(), right.sexp()].sexp_move()
            }
            Unary(unary, expression, _, _) => [unary.sexp(), expression.sexp()].sexp_move(),
            Index(array, index, _, _) => ["[]".sexp(), array.sexp(), index.sexp()].sexp_move(),
            Length(array, _) => ["length".sexp(), array.sexp()].sexp_move(),
            Dot(_, receiver, symbol, _, _) => {
                [".".sexp(), receiver.sexp(), symbol.sexp()].sexp_move()
            }
            New(variable, _) => ["new".sexp(), variable.sexp()].sexp_move(),
            Call(call) => call.sexp(),
        }
    }
}

impl<T> Serialize for ast::Declaration<T> {
    fn sexp(&self) -> Sexp {
        match self {
            ast::Declaration::Multiple(multiple) => multiple.sexp(),
            ast::Declaration::Single(single) => single.sexp(),
        }
    }
}

impl<T> Serialize for ast::MultipleDeclaration<T> {
    fn sexp(&self) -> Sexp {
        [self.names.sexp(), self.r#type.sexp()].sexp_move()
    }
}

impl<T> Serialize for ast::SingleDeclaration<T> {
    fn sexp(&self) -> Sexp {
        [self.name.sexp(), self.r#type.sexp()].sexp_move()
    }
}

impl<T> Serialize for ast::Call<T> {
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

impl<T> Serialize for ast::Statement<T> {
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

impl<T> Serialize for ast::Variable<T> {
    fn sexp(&self) -> Sexp {
        match self.generics.as_ref() {
            None => self.name.sexp(),
            Some(generics) => [self.name.sexp(), generics.sexp()].sexp_move(),
        }
    }
}

impl Serialize for ast::Identifier {
    fn sexp(&self) -> Sexp {
        self.symbol.sexp()
    }
}
