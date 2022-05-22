use std::fmt::Write as _;

use crate::data::r#type;
use crate::data::symbol;
use crate::data::symbol::Symbol;

pub fn class_size(class: &Symbol) -> Symbol {
    symbol::intern(&format!("_I_size_{}", escape(class)))
}

pub fn class_virtual_table(class: &Symbol) -> Symbol {
    symbol::intern(&format!("_I_vt_{}", escape(class)))
}

#[allow(unused)]
pub fn class_initialization(class: &Symbol) -> Symbol {
    symbol::intern(&format!("_I_init_{}", escape(class)))
}

#[allow(unused)]
pub fn global(name: &Symbol, r#type: &r#type::Expression) -> Symbol {
    let mut mangled = format!("_I_g_{}_", escape(name));
    mangled.push('_');
    mangle_type(r#type, &mut mangled);
    symbol::intern(mangled)
}

#[allow(unused)]
pub fn method(
    class: &Symbol,
    name: &Symbol,
    parameters: &[r#type::Expression],
    returns: &[r#type::Expression],
) -> Symbol {
    let mut mangled = format!("_M_{}_", escape(class));
    mangle_function(name, parameters, returns, &mut mangled);
    symbol::intern(mangled)
}

pub fn function(
    name: &Symbol,
    parameters: &[r#type::Expression],
    returns: &[r#type::Expression],
) -> Symbol {
    let mut mangled = String::new();
    mangle_function(name, parameters, returns, &mut mangled);
    symbol::intern(mangled)
}

fn mangle_function(
    name: &Symbol,
    parameters: &[r#type::Expression],
    returns: &[r#type::Expression],
    mangled: &mut String,
) {
    write!(mangled, "_I{}_", escape(name)).unwrap();

    match returns {
        [] => mangled.push('p'),
        [r#type] => {
            mangle_type(r#type, mangled);
        }
        types => {
            mangled.push('t');
            write!(mangled, "{}", types.len()).unwrap();
            for r#type in types {
                mangle_type(r#type, mangled);
            }
        }
    }

    for parameter in parameters {
        mangle_type(parameter, mangled);
    }
}

fn mangle_type(r#type: &r#type::Expression, mangled: &mut String) {
    match r#type {
        r#type::Expression::Any => panic!("[INTERNAL ERROR]: any type in IR"),
        r#type::Expression::Integer => mangled.push('i'),
        r#type::Expression::Boolean => mangled.push('b'),
        r#type::Expression::Class(class) => {
            mangled.push('o');
            write!(mangled, "{}", symbol::resolve(*class).len()).unwrap();
            write!(mangled, "{}", escape(class)).unwrap();
        }
        r#type::Expression::Array(r#type) => {
            mangled.push('a');
            mangle_type(&*r#type, mangled);
        }
    }
}

fn escape(symbol: &Symbol) -> String {
    symbol::resolve(*symbol).replace('_', "__")
}
