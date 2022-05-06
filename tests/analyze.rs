use std::io::Write as _;
use std::process;

use xic::api::analyze::display;
use xic::api::analyze::LiveVariables;
use xic::asm;
use xic::data::asm::Function;
use xic::data::operand::Label;
use xic::data::operand::Temporary;
use xic::data::symbol;

macro_rules! assembly {
    ($function:ident: $arguments:tt -> $returns:tt; $($tt:tt)*) => {
        {
            let enter = Label::Fixed(symbol::intern_static("enter"));
            let exit = Label::Fixed(symbol::intern_static("exit"));

            let mut statements = assembly!($($tt)*);
            statements.insert(0, asm!((enter:)));
            statements.push(asm!((exit:)));
            statements.push(asm!((ret<$returns>)));

            Function::<Temporary> {
                name: symbol::intern_static(stringify!($function)),
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
            $(let $temporary = Temporary::Fixed(symbol::intern_static(stringify!($temporary)));)*
            {
                assembly!($($tt)*)
            }
        }
    };
    (labels: $($label:ident),+ $(,)? ; $($tt:tt)*) => {
        {
            $(let $label = Label::Fixed(symbol::intern_static(stringify!($label)));)*
            {
                assembly!($($tt)*)
            }
        }
    };
    ($($statement:tt)*) => {
        vec![$(asm!($statement)),*]
    };
}

macro_rules! live_variables {
    ($function:ident $($tt:tt)*) => {
        #[test]
        fn $function() {
            let function = assembly!($function $($tt)*);
            let (live_variables, live_ranges) = live(&function);
            insta::assert_display_snapshot!(live_variables);
            insta::assert_display_snapshot!(live_ranges);
        }
    }
}

fn live(function: &Function<Temporary>) -> (String, String) {
    let cfg = xic::api::construct_cfg(function);

    let mut graph = process::Command::new("graph-easy")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .arg("-")
        .spawn()
        .unwrap();

    write!(
        &mut graph.stdin.as_mut().unwrap(),
        "{}",
        display::<LiveVariables<Function<Temporary>>, _>(&cfg)
    )
    .unwrap();

    let live_variables = graph.wait_with_output().unwrap();
    if !live_variables.status.success() {
        panic!("Failed to generate diagram from .dot file");
    }
    let live_ranges = xic::api::analyze::LiveRanges::new(&cfg);

    (
        String::from_utf8(live_variables.stdout).unwrap(),
        live_ranges.to_string(),
    )
}

live_variables! {
    smoke: 0 -> 0;
}

live_variables! {
    smoke_move: 0 -> 1;
    temporaries: x;
    (mov rax, x)
}
