use xic::analyze::analyze_default;
use xic::analyze::display;
use xic::analyze::CopyPropagation;
use xic::data::asm::Function;
use xic::data::operand::Temporary;

macro_rules! copy_propagation {
    ($function:ident $($tt:tt)*) => {
        #[test]
        fn $function() -> anyhow::Result<()> {
            let function = asm_function!($function $($tt)*);
            insta::assert_display_snapshot!(copy_propagation(function)?);
            Ok(())
        }
    }
}

fn copy_propagation(function: Function<Temporary>) -> anyhow::Result<String> {
    let cfg = xic::api::construct_cfg(function);
    let copy_propagation = analyze_default::<CopyPropagation, _>(&cfg);
    super::super::graph(display(&copy_propagation, &cfg))
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
    clobbered_across_call: 0 -> 0;
    temporaries: a;
    labels: black_box;
    (mov rax, a)
    (mov rcx, a)
    (mov rdx, a)
    (mov rsi, a)
    (mov rdi, a)
    (mov r8, a)
    (mov r9, a)
    (mov r10, a)
    (mov r11, a)
    (call<0, 0> black_box)
}

copy_propagation! {
    clobbered_across_hul: 0 -> 0;
    temporaries: a;
    (mov rax, a)
    (mov rdx, a)
    (ihul 1)
}

copy_propagation! {
    clobbered_across_div: 0 -> 0;
    temporaries: a;
    (mov rax, a)
    (mov rdx, a)
    (idiv 1)
}

copy_propagation! {
    clobbered_across_mod: 0 -> 0;
    temporaries: a;
    (mov rax, a)
    (mov rdx, a)
    (imod 1)
}

copy_propagation! {
    clobbered_across_cqo: 0 -> 0;
    temporaries: a;
    (mov rax, a)
    (mov rdx, a)
    (cqo)
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
    temporaries: a, b, c, d;
    labels: exit, branch, fallthrough;
        (cmp a, b)
        (mov d, a)
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
    temporaries: a, b, c;
    labels: exit, branch, fallthrough;
        (cmp a, b)
        (mov c, a)
        (jne branch)
    (fallthrough:)
        (mov a, b)
        (jmp exit)
    (branch:)
        (mov a, b)
        (jmp exit)
}
