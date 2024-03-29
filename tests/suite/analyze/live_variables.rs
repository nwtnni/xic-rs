use xic::analyze::analyze_default;
use xic::analyze::display;
use xic::analyze::LiveRanges;
use xic::analyze::LiveVariables;
use xic::data::asm::Function;
use xic::data::operand::Scale::_8;
use xic::data::operand::Temporary;

macro_rules! live_variables {
    ($function:ident $($tt:tt)*) => {
        #[test]
        fn $function() -> anyhow::Result<()> {
            let function = asm_function!($function $($tt)*);
            let (live_variables, live_ranges) = live(function)?;
            insta::assert_display_snapshot!(live_variables);
            insta::assert_display_snapshot!(live_ranges);
            Ok(())
        }
    }
}

fn live(function: Function<Temporary>) -> anyhow::Result<(String, String)> {
    let cfg = xic::api::construct_cfg(function);
    let live_variables = analyze_default::<LiveVariables<_>, _>(&cfg);
    let annotated_cfg = super::super::graph(display(&live_variables, &cfg))?;
    let annotated_assembly = LiveRanges::new(&live_variables, cfg).to_string();
    Ok((annotated_cfg, annotated_assembly))
}

live_variables! {
    smoke: 0 -> 0;
}

live_variables! {
    smoke_move: 0 -> 1;
    temporaries: x;
    (mov rax, x)
}

live_variables! {
    pass_one_receieve_one: 0 -> 1;
    labels: black_box;
    temporaries: x;
    (mov x, 5)
    (mov rdi, x)
    (call<1, 1> black_box)
    (mov x, rax)
    (mov rax, x)
}

live_variables! {
    clobber_across_call: 0 -> 2;
    labels: clobber;
    temporaries: x, y;
    (mov x, 0)
    (mov y, 1)
    (call<0, 0> clobber)
    (mov rax, x)
    (mov rdx, y)
}

live_variables! {
    clobber_across_div: 0 -> 2;
    temporaries: x, y;
    (mov x, 1)
    (mov y, 2)
    (mov rax, x)
    (cqo)
    (idiv 5)
    (mov rdx, y)
}

live_variables! {
    clobber_across_hul: 0 -> 2;
    temporaries: x, y;
    (mov x, 1)
    (mov y, 2)
    (mov rax, x)
    (ihul 5)
    (mov rax, y)
}

live_variables! {
    clobber_across_mod: 0 -> 2;
    temporaries: x, y;
    (mov x, 1)
    (mov y, 2)
    (mov rax, x)
    (cqo)
    (imod 5)
    (mov rax, y)
}

live_variables! {
    clobber_across_div_mod: 0 -> 2;
    temporaries: x, y;
    (mov x, 1)
    (mov y, 2)
    (mov rax, x)
    (cqo)
    (idiv 1)
    (cqo)
    (imod 10)
    (mov rax, y)
}

live_variables! {
    redundant_moves: 0 -> 1;
    temporaries: useless;
    (mov useless, 0)
    (mov useless, 1)
    (mov useless, 2)
    (mov useless, 3)
    (mov rax, useless)
}

live_variables! {
    propagate_liveness_move: 0 -> 1;
    temporaries: x, y, z;
    (mov x, 0)
    (mov y, x)
    (mov z, y)
    (mov rdx, z)
    (mov rax, rdx)
}

live_variables! {
    propagate_liveness_neg: 0 -> 1;
    temporaries: x;
    (mov x, 1)
    (neg x)
    (neg x)
    (neg x)
    (mov rax, x)
}

live_variables! {
    propagate_liveness_memory: 0 -> 1;
    temporaries: x;
    (mov x, 1)
    (mov x, [x])
    (mov rax, x)
}

live_variables! {
    propagate_liveness_memory_two: 0 -> 1;
    temporaries: x, y;
    (mov x, 1)
    (mov y, 2)
    (mov x, [x + y * _8])
    (mov rax, x)
}

live_variables! {
    propagate_liveness_memory_tree: 0 -> 1;
    temporaries: a, b, c, d, l, r;
    (mov a, 0)
    (mov b, 1)
    (mov c, 2)
    (mov d, 3)
    (mov a, [a + 8])
    (mov b, [b + c])
    (mov c, [c * _8 + 8])
    (mov d, [d])
    (mov l, [c + d + 8])
    (mov r, [a + b * _8])
    (mov rax, [l + r * _8 + 16])
}

live_variables! {
    everything_is_meaningless: 0 -> 0;
    temporaries: x, y, z;
    (mov x, 1)
    (mov y, 2)
    (add y, x)
    (mov z, 5)
    (sub x, y)
    (mov rax, [x + z])
    (ihul 2)
    (neg rax)
    (nop)
    (mov z, rdx)
}

live_variables! {
    everything_is_meaningless_except_div_mod: 0 -> 0;
    temporaries: x, y;
    (mov x, 1)
    (mov y, 2)
    (add y, x)
    (mov rax, y)
    (cqo)
    (idiv 2)
    (mov x, rax)
    (add x, 1)
    (add x, y)
}

live_variables! {
    everything_is_meaningless_except_call: 0 -> 0;
    temporaries: x, y;
    labels: black_box;
    (mov x, 1)
    (mov y, 2)
    (add x, y)
    (add y, x)
    (mov rdi, x)
    (mov rsi, y)
    (call<2, 0> black_box)
    (add x, y)
    (add y, x)
}

live_variables! {
    everything_is_meaningless_except_cmp: 0 -> 0;
    temporaries: x, y;
    labels: exit, fallthrough;
        (mov x, 1)
        (mov y, 2)
        (cmp x, y)
        (add x, 1)
        (je exit)
    (fallthrough:)
        (add y, 1)
        (jmp exit)
}

live_variables! {
    everything_is_meaningless_except_memory_write: 0 -> 0;
    temporaries: x, y;
    (mov x, 1)
    (mov y, 2)
    (mov [x + y], 5)
    (add x, 1)
    (add y, 1)
}

live_variables! {
    div_mod_cant_save_you_all_from_death: 0 -> 0;
    temporaries: x, y, useless;
    (mov useless, 0)
    (mov x, 1)
    (mov y, 2)
    (add y, x)
    (mov rax, y)
    (add useless, 5)
    (cqo)
    (idiv 2)
    (mov x, rax)
    (add x, 1)
    (add x, y)
    (add useless, 5)
}

live_variables! {
    call_cant_save_you_all_from_death: 0 -> 0;
    temporaries: x, y, useless;
    labels: black_box;
    (mov useless, 0)
    (mov x, 1)
    (mov y, 2)
    (add x, y)
    (add y, x)
    (add useless, 5)
    (mov rdi, x)
    (mov rsi, y)
    (call<2, 0> black_box)
    (add x, y)
    (add y, x)
    (neg useless)
}

live_variables! {
    cmp_cant_save_you_all_from_death: 0 -> 0;
    temporaries: x, y, useless;
    labels: exit, fallthrough;
        (mov useless, 0)
        (mov x, 1)
        (add useless, 5)
        (mov y, 2)
        (cmp x, y)
        (add x, 1)
        (and useless, 1)
        (je exit)
    (fallthrough:)
        (or useless, 1)
        (add y, 1)
        (jmp exit)
}

live_variables! {
    memory_write_cant_save_you_all_from_death: 0 -> 0;
    temporaries: x, y, useless;
    (mov x, 1)
    (mov y, 2)
    (mov useless, 0)
    (add useless, x)
    (mov [x + y], 5)
    (add x, 1)
    (add y, 1)
    (add useless, y)
}

live_variables! {
    merge: 0 -> 1;
    temporaries: a, b, c, d;
    labels: exit, branch, fallthrough;
        (jne branch)
    (fallthrough:)
        (mov d, c)
        (mov rax, b)
        (jmp exit)
    (branch:)
        (mov c, d)
        (mov rax, a)
        (jmp exit)
}
