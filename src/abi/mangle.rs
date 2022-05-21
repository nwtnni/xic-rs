use std::fmt::Write as _;

use crate::data::r#type;
use crate::data::symbol;

#[allow(unused)]
pub fn mangle_global(name: &str, r#type: &r#type::Expression) -> String {
    let mut mangled = format!("_I_g_{}_", escape(name));
    mangled.push('_');
    mangle_type(r#type, &mut mangled);
    mangled
}

#[allow(unused)]
pub fn mangle_method(
    class: &str,
    name: &str,
    parameters: &[r#type::Expression],
    returns: &[r#type::Expression],
) -> String {
    let mut mangled = mangle_function(name, parameters, returns);
    write!(mangled, "_M_").unwrap();
    write!(mangled, "{}", escape(class)).unwrap();
    mangled
}

pub fn mangle_function(
    name: &str,
    parameters: &[r#type::Expression],
    returns: &[r#type::Expression],
) -> String {
    let mut mangled = format!("_I{}_", escape(name));

    match returns {
        [] => mangled.push('p'),
        [r#type] => {
            mangle_type(r#type, &mut mangled);
        }
        types => {
            mangled.push('t');
            write!(&mut mangled, "{}", types.len()).unwrap();
            for r#type in types {
                mangle_type(r#type, &mut mangled);
            }
        }
    }

    for parameter in parameters {
        mangle_type(parameter, &mut mangled);
    }

    mangled
}

fn mangle_type(r#type: &r#type::Expression, mangled: &mut String) {
    match r#type {
        r#type::Expression::Any => panic!("[INTERNAL ERROR]: any type in IR"),
        r#type::Expression::Integer => mangled.push('i'),
        r#type::Expression::Boolean => mangled.push('b'),
        r#type::Expression::Class(class) => {
            let class = symbol::resolve(*class);
            mangled.push('o');
            write!(mangled, "{}", class.len()).unwrap();
            write!(mangled, "{}", escape(class)).unwrap();
        }
        r#type::Expression::Array(r#type) => {
            mangled.push('a');
            mangle_type(&*r#type, mangled);
        }
    }
}

fn escape(string: &str) -> String {
    string.replace('_', "__")
}
