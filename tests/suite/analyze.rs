macro_rules! lir_function {
    ($function:ident: $arguments:tt -> $returns:tt; $($tt:tt)*) => {
        {
            let enter = xic::data::operand::Label::Fixed(xic::data::symbol::intern_static("enter"));
            let exit = xic::data::operand::Label::Fixed(xic::data::symbol::intern_static("exit"));

            let mut statements = lir_function!($($tt)*);

            statements.insert(0, xic::lir!((LABEL enter)));
            statements.push(xic::lir!((LABEL exit)));

            xic::data::lir::Function::<xic::data::lir::Fallthrough> {
                name: xic::data::symbol::intern_static(stringify!($function)),
                statements,
                arguments: $arguments,
                returns: $returns,
                enter,
                exit,
            }
        }
    };
    (temporaries: $($temporary:ident),+ $(,)? ; $($tt:tt)*) => {
        {
            $(
                let $temporary = xic::data::operand::Temporary::Fixed(
                    xic::data::symbol::intern_static(stringify!($temporary))
                );
            )*
            {
                lir_function!($($tt)*)
            }
        }
    };
    (labels: $($label:ident),+ $(,)? ; $($tt:tt)*) => {
        {
            $(
                let $label = xic::data::operand::Label::Fixed(
                    xic::data::symbol::intern_static(stringify!($label))
                );
            )*
            {
                lir_function!($($tt)*)
            }
        }
    };
    ($($statement:tt)*) => {
        vec![$(xic::lir!($statement)),*]
    };
}

macro_rules! asm_function {
    ($function:ident: $arguments:tt -> $returns:tt; $($tt:tt)*) => {
        {
            let enter = xic::data::operand::Label::Fixed(xic::data::symbol::intern_static("enter"));
            let exit = xic::data::operand::Label::Fixed(xic::data::symbol::intern_static("exit"));

            let mut statements = asm_function!($($tt)*);

            statements.insert(0, xic::asm!((enter:)));
            statements.push(xic::asm!((exit:)));
            statements.push(xic::asm!((ret<$returns>)));

            xic::data::asm::Function::<xic::data::operand::Temporary> {
                name: xic::data::symbol::intern_static(stringify!($function)),
                statements,
                arguments: $arguments,
                returns: $returns,
                enter,
                exit,
            }
        }
    };
    (temporaries: $($temporary:ident),+ $(,)? ; $($tt:tt)*) => {
        {
            $(
                let $temporary = xic::data::operand::Temporary::Fixed(
                    xic::data::symbol::intern_static(stringify!($temporary))
                );
            )*
            {
                asm_function!($($tt)*)
            }
        }
    };
    (labels: $($label:ident),+ $(,)? ; $($tt:tt)*) => {
        {
            $(
                let $label = xic::data::operand::Label::Fixed(
                    xic::data::symbol::intern_static(stringify!($label))
                );
            )*
            {
                asm_function!($($tt)*)
            }
        }
    };
    ($($statement:tt)*) => {
        vec![$(xic::asm!($statement)),*]
    };
}

#[path = "analyze/constant_propagation.rs"]
mod constant_propagation;

#[path = "analyze/copy_propagation.rs"]
mod copy_propagation;

#[path = "analyze/lazy_code_motion.rs"]
mod lazy_code_motion;

#[path = "analyze/live_variables.rs"]
mod live_variables;
