use xic::api::analyze::analyze;
use xic::api::analyze::display;
use xic::api::analyze::ConstantPropagation;
use xic::data::asm::Function;
use xic::data::operand::Temporary;

macro_rules! constant_propagation {
    ($function:ident $($tt:tt)*) => {
        #[test]
        fn $function() {
            let function = assembly!($function $($tt)*);
            let copy_propagation = constant_propagation(function);
            insta::assert_display_snapshot!(copy_propagation);
        }
    }
}

fn constant_propagation(function: Function<Temporary>) -> String {
    let cfg = xic::api::construct_cfg(function);
    let copy_propagation = analyze::<ConstantPropagation, _>(&cfg);
    let annotated_cfg = super::super::graph(display(&copy_propagation, &cfg));
    annotated_cfg
}

constant_propagation! {
    smoke: 0 -> 0;
    temporaries: a;
    (mov a, 0)
}

constant_propagation! {
    overwrite: 0 -> 0;
    temporaries: a;
    (mov a, 0)
    (mov a, 1)
}

constant_propagation! {
    propagate: 0 -> 0;
    temporaries: a, b, c;
    (mov a, 0)
    (mov b, a)
    (mov c, b)
}

constant_propagation! {
    propagate_add: 0 -> 0;
    temporaries: a, b;
    (mov a, 1)
    (mov b, 2)
    (add b, a)
}

constant_propagation! {
    propagate_sub: 0 -> 0;
    temporaries: a;
    (mov a, 5)
    (sub a, 2)
}

constant_propagation! {
    propagate_mod: 0 -> 0;
    temporaries: a;
    (mov a, 5)
    (mov rax, 3)
    (cqo)
    (imod a)
}

constant_propagation! {
    clobbered_across_call: 0 -> 0;
    labels: black_box;
    (mov rax, 0)
    (mov rcx, 0)
    (mov rdx, 0)
    (mov rsi, 0)
    (mov rdi, 0)
    (mov r8, 0)
    (mov r9, 0)
    (mov r10, 0)
    (mov r11, 0)
    (call<0, 0> black_box)
}

constant_propagation! {
    clobbered_across_hul: 0 -> 0;
    temporaries: a;
    (mov rax, 1)
    (mov rdx, 2)
    (ihul a)
}

constant_propagation! {
    clobbered_across_div: 0 -> 0;
    temporaries: a;
    (mov rax, 1)
    (mov rdx, 2)
    (idiv a)
}

constant_propagation! {
    clobbered_across_mod: 0 -> 0;
    temporaries: a;
    (mov rax, 1)
    (mov rdx, 2)
    (imod a)
}

constant_propagation! {
    clobbered_across_cqo: 0 -> 0;
    (mov rax, 1)
    (mov rdx, 2)
    (cqo)
}

constant_propagation! {
    defined_twice_different: 0 -> 0;
    temporaries: a;
    labels: exit, branch, fallthrough;
        (jne branch)
    (fallthrough:)
        (mov a, 1)
        (jmp exit)
    (branch:)
        (mov a, 2)
        (jmp exit)
}

constant_propagation! {
    defined_twice_identical: 0 -> 0;
    temporaries: a;
    labels: exit, branch, fallthrough;
        (jne branch)
    (fallthrough:)
        (mov a, 1)
        (jmp exit)
    (branch:)
        (mov a, 1)
        (jmp exit)
}

constant_propagation! {
    merge_many: 0 -> 0;
    temporaries: a, b, c, d, e;
    labels: exit, fallthrough, branch1, branch2, branch3, merge;
        (mov a, 1)
        (jne branch3)
    (fallthrough:)
        (mov b, 2)
        (jne branch2)
    (branch1:)
        (mov c, 1)
        (mov d, 3)
        (add d, b)
        (mov e, 4)
        (jmp merge)
    (branch2:)
        (mov c, 2)
        (mov d, 7)
        (sub d, b)
        (jmp merge)
    (branch3:)
        (mov c, 1)
        (mov d, 5)
        (jmp merge)
    (merge:)
        (jmp exit)
}
