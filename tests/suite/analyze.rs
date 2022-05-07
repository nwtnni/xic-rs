macro_rules! assembly {
    ($function:ident: $arguments:tt -> $returns:tt; $($tt:tt)*) => {
        {
            let enter = xic::data::operand::Label::Fixed(xic::data::symbol::intern_static("enter"));
            let exit = xic::data::operand::Label::Fixed(xic::data::symbol::intern_static("exit"));

            use xic::asm;

            let mut statements = assembly!($($tt)*);

            statements.insert(0, asm!((enter:)));
            statements.push(asm!((exit:)));
            statements.push(asm!((ret<$returns>)));

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
                assembly!($($tt)*)
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
                assembly!($($tt)*)
            }
        }
    };
    ($($statement:tt)*) => {
        vec![$(asm!($statement)),*]
    };
}

#[path = "analyze/live_variables.rs"]
mod live_variables;

#[path = "analyze/constant_propagation.rs"]
mod constant_propagation;

#[path = "analyze/copy_propagation.rs"]
mod copy_propagation;
