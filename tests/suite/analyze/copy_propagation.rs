use xic::api::analyze::analyze;
use xic::api::analyze::display;
use xic::api::analyze::CopyPropagation;
use xic::data::asm::Function;
use xic::data::operand::Temporary;

macro_rules! copy_propagation {
    ($function:ident $($tt:tt)*) => {
        #[test]
        fn $function() {
            let function = assembly!($function $($tt)*);
            let copy_propagation = copy_propagation(function);
            insta::assert_display_snapshot!(copy_propagation);
        }
    }
}

fn copy_propagation(function: Function<Temporary>) -> String {
    let cfg = xic::api::construct_cfg(function);
    let copy_propagation = analyze::<CopyPropagation, _>(&cfg);
    let annotated_cfg = super::super::graph(display(&copy_propagation, &cfg));
    annotated_cfg
}

copy_propagation! {
    redundant_copies: 0 -> 0;
    temporaries: a, b, c, d, e;
    (mov b, a)
    (mov c, b)
    (mov d, c)
    (mov e, d)
}

copy_propagation! {
    overwritten_tree: 0 -> 0;
    temporaries: a, b, c, d;
    (mov a, b)
    (mov c, a)
    (mov d, a)
    (mov b, 1)
}

copy_propagation! {
    partially_overwritten_tree: 0 -> 0;
    temporaries: a, b, c, d, e;
    (mov a, b)
    (mov d, a)
    (mov c, b)
    (mov e, c)
    (mov a, 1)
}

copy_propagation! {
    defined_twice_different: 0 -> 0;
    temporaries: a, b, c;
    labels: exit, branch, fallthrough;
    (cmp a, b)
    (jne branch)
    (fallthrough:)
    (mov a, b)
    (jmp exit)
    (branch:)
    (mov a, c)
    (jmp exit)
}

copy_propagation! {
    defined_twice_identical: 0 -> 0;
    temporaries: a, b;
    labels: exit, branch, fallthrough;
    (cmp a, b)
    (jne branch)
    (fallthrough:)
    (mov a, b)
    (jmp exit)
    (branch:)
    (mov a, b)
    (jmp exit)
}
