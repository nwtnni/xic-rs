use std::fmt;
use std::fmt::Write as _;

use crate::data::ast;
use crate::data::r#type;
use crate::data::symbol;
use crate::data::symbol::Symbol;

pub fn template<T>(name: &Symbol, generics: &[ast::Type<T>]) -> Symbol {
    let mut mangled = String::new();
    mangle_template(name, generics, &mut mangled).unwrap();
    symbol::intern(mangled)
}

pub fn class_size(class: &Symbol) -> Symbol {
    symbol::intern(&format!("_I_size_{}", escape(class)))
}

pub fn class_virtual_table(class: &Symbol) -> Symbol {
    symbol::intern(&format!("_I_vt_{}", escape(class)))
}

pub fn class_initialization(class: &Symbol) -> Symbol {
    symbol::intern(&format!("_I_init_{}", escape(class)))
}

pub fn global_initialization<'a, 'b, I>(initialization: I) -> Symbol
where
    I: IntoIterator<Item = (&'a Symbol, &'b r#type::Expression)>,
{
    let mut mangled = String::from("_I_init_global");
    for (name, r#type) in initialization {
        write!(&mut mangled, "_{}", escape(name)).unwrap();
        mangle_type(r#type, &mut mangled).unwrap();
    }
    symbol::intern(mangled)
}

pub fn global(name: &Symbol, r#type: &r#type::Expression) -> Symbol {
    let mut mangled = format!("_I_global_{}_", escape(name));
    mangle_type(r#type, &mut mangled).unwrap();
    symbol::intern(mangled)
}

pub fn method(
    class: &Symbol,
    name: &Symbol,
    parameters: &[r#type::Expression],
    returns: &[r#type::Expression],
) -> Symbol {
    let mut mangled = format!("_I_{}_", escape(class));
    mangle_function(name, parameters, returns, &mut mangled).unwrap();
    symbol::intern(mangled)
}

pub fn function(
    name: &Symbol,
    parameters: &[r#type::Expression],
    returns: &[r#type::Expression],
) -> Symbol {
    let mut mangled = String::from("_I");
    mangle_function(name, parameters, returns, &mut mangled).unwrap();
    symbol::intern(mangled)
}

fn mangle_function(
    name: &Symbol,
    parameters: &[r#type::Expression],
    returns: &[r#type::Expression],
    mangled: &mut String,
) -> fmt::Result {
    write!(mangled, "{}_", escape(name))?;

    match returns {
        [] => mangled.push('p'),
        [r#type] => {
            mangle_type(r#type, mangled)?;
        }
        types => {
            mangled.push('t');
            write!(mangled, "{}", types.len())?;
            for r#type in types {
                mangle_type(r#type, mangled)?;
            }
        }
    }

    for parameter in parameters {
        mangle_type(parameter, mangled)?;
    }

    Ok(())
}

fn mangle_type(r#type: &r#type::Expression, mangled: &mut String) -> fmt::Result {
    match r#type {
        r#type::Expression::Any | r#type::Expression::Null | r#type::Expression::Function(_, _) => {
            panic!("[INTERNAL ERROR]: `{}` type in IR", r#type)
        }
        r#type::Expression::Integer => mangled.push('i'),
        r#type::Expression::Boolean => mangled.push('b'),
        r#type::Expression::Class(class) => {
            let name = escape(class);
            write!(mangled, "o{}{}", name.len(), name)?;
        }
        r#type::Expression::Array(r#type) => {
            mangled.push('a');
            mangle_type(&*r#type, mangled)?;
        }
    }

    Ok(())
}

fn mangle_template<T>(
    name: &Symbol,
    generics: &[ast::Type<T>],
    mangled: &mut String,
) -> Result<(), fmt::Error> {
    let name = escape(name);

    write!(mangled, "t{}{}{}", name.len(), name, generics.len())?;

    for generic in generics {
        match generic {
            // Note: this branch guarantees the following invariant:
            //
            // Mangling a recursive template type (1) step-by-step in postorder and
            // (2) all at once produces the same output. These two cases are encountered
            // in the loading and monormophizing passes, respectively. For example,
            // consider the following type:
            //
            // ```
            // A::<B::<C>, D>
            //
            // (1) A::<o7t1B1o1C, o1D> (2) t1A2o7t1B1o1Co1D
            //     t1A2o7t1B1o1Co1D
            // ```
            //
            // We need to make sure the extra `o7` prefix is added in case (2).
            class @ ast::Type::Class(ast::Variable {
                name: _,
                generics: Some(_),
                span: _,
            }) => {
                let mut buffer = String::new();
                mangle_type_ast(class, &mut buffer)?;
                write!(mangled, "o{}{}", buffer.len(), buffer)?;
            }
            generic => mangle_type_ast(generic, mangled)?,
        }
    }

    Ok(())
}

fn mangle_type_ast<T>(r#type: &ast::Type<T>, mangled: &mut String) -> Result<(), fmt::Error> {
    match r#type {
        ast::Type::Int(_) => mangled.push('i'),
        ast::Type::Bool(_) => mangled.push('b'),
        ast::Type::Class(ast::Variable {
            name,
            generics: None,
            span: _,
        }) => {
            let name = escape(&name.symbol);
            write!(mangled, "o{}{}", name.len(), name)?;
        }
        ast::Type::Class(ast::Variable {
            name,
            generics: Some(generics),
            span: _,
        }) => {
            mangle_template(&name.symbol, generics, mangled)?;
        }
        ast::Type::Array(r#type, _, _) => {
            mangled.push('a');
            mangle_type_ast(&*r#type, mangled)?;
        }
    }

    Ok(())
}

fn escape(symbol: &Symbol) -> String {
    symbol::resolve(*symbol)
        .replace('\'', "_")
        .replace('_', "__")
}
