use xic::api::analyze::analyze;
use xic::api::analyze::display;
use xic::api::analyze::AnticipatedExpressions;
use xic::api::analyze::AvailableExpressions;
use xic::api::analyze::Earliest;
use xic::api::analyze::Latest;
use xic::api::analyze::PostponableExpressions;
use xic::api::analyze::UsedExpressions;
use xic::data::lir::Fallthrough;
use xic::data::lir::Function;

macro_rules! partial_redundancy_elimination {
    ($function:ident $($tt:tt)*) => {
        #[test]
        fn $function() {
            let function = lir_function!($function $($tt)*);
            let (anticipated, available, earliest, postponable, latest, used) = partial_redundancy_elimination(function);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_anticipated"), anticipated);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_available"), available);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_earliest"), earliest);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_postponable"), postponable);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_latest"), latest);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_used"), used);
        }
    }
}

fn partial_redundancy_elimination(
    function: Function<Fallthrough>,
) -> (String, String, String, String, String, String) {
    let cfg = xic::api::construct_cfg(function);

    let solution = analyze::<AnticipatedExpressions, _>(&cfg);
    let anticipated = super::super::graph(display(&solution, &cfg));

    let solution = analyze::<AvailableExpressions<_>, _>(&cfg);
    let available = super::super::graph(display(&solution, &cfg));

    let solution = analyze::<Earliest<_>, _>(&cfg);
    let earliest = super::super::graph(display(&solution, &cfg));

    let solution = analyze::<PostponableExpressions<_>, _>(&cfg);
    let postponable = super::super::graph(display(&solution, &cfg));

    let solution = analyze::<Latest<_>, _>(&cfg);
    let latest = super::super::graph(display(&solution, &cfg));

    let solution = analyze::<UsedExpressions<_>, _>(&cfg);
    let used = super::super::graph(display(&solution, &cfg));

    (anticipated, available, earliest, postponable, latest, used)
}

partial_redundancy_elimination! {
    used_on_one_branch: 0 -> 0;
    temporaries: a, b, c;
    labels: branch, fallthrough, merge, exit;
        (MOVE (TEMP b) (CONST 0))
        (CJUMP (EQ (CONST 0) (CONST 0)) branch)
    (LABEL fallthrough)
        (MOVE (TEMP a) (ADD (TEMP b) (CONST 1)))
        (JUMP merge)
    (LABEL branch)
        (MOVE (TEMP a) (CONST 0))
        (JUMP merge)
    (LABEL merge)
        (MOVE (TEMP c) (ADD (TEMP b) (CONST 1)))
        (JUMP exit)
}

partial_redundancy_elimination! {
    basic_loop: 0 -> 0;
    temporaries: a, b, c;
    labels: r#loop, split, fallthrough, exit;
        (MOVE (TEMP a) (CONST 0))
        (JUMP r#loop)
    (LABEL r#loop)
        (MOVE (TEMP c) (ADD (TEMP a) (TEMP b)))
        (CJUMP (EQ (CONST 0) (CONST 0)) split)
    (LABEL fallthrough)
        (JUMP exit)
    (LABEL split)
        (JUMP r#loop)
}

partial_redundancy_elimination! {
    // https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.92.4197&rep=rep1&type=pdf
    //
    // Note: loop between _12 and _13 omitted, as there's nothing interesting there.
    knoop_ruthing_steffen: 0 -> 0;
    temporaries: a, b, c, x, y;
    labels: _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _14, _15, _16, _17, _18, exit;
    (LABEL _1)
        (CJUMP (EQ (CONST 0) (CONST 0)) _2)
    (LABEL _4)
        (JUMP _5)
    (LABEL _2)
        (MOVE (TEMP a) (TEMP c))
        (JUMP _3)
    (LABEL _3)
        (MOVE (TEMP x) (ADD (TEMP a) (TEMP b)))
        (JUMP _5)
    (LABEL _5)
        (CJUMP (EQ (CONST 0) (CONST 0)) _6)
    (LABEL _7)
        (JUMP _18)
    (LABEL _6)
        (CJUMP (EQ (CONST 0) (CONST 0)) _8)

    (LABEL _9)
        (JUMP _12)
    (LABEL _12)
        (CJUMP (EQ (CONST 0) (CONST 0)) _17)
    (LABEL _15)
        (MOVE (TEMP y) (ADD (TEMP a) (TEMP b)))
        (JUMP _16)
    (LABEL _17)
        (MOVE (TEMP x) (ADD (TEMP a) (TEMP b)))
        (JUMP _18)

    (LABEL _8)
        (JUMP _11)
    (LABEL _11)
        (CJUMP (EQ (CONST 0) (CONST 0)) _14)
    (LABEL _10)
        (MOVE (TEMP y) (ADD (TEMP a) (TEMP b)))
        (JUMP _11)
    (LABEL _14)
        (JUMP _16)

    (LABEL _16)
        (MOVE (TEMP x) (ADD (TEMP a) (TEMP b)))
        (JUMP _18)
    (LABEL _18)
        (JUMP exit)
}

partial_redundancy_elimination! {
    call_argument: 0 -> 0;
    labels: black_box;
    (CALL (NAME black_box) 0 (ADD (CONST 1) (CONST 2)))
}

partial_redundancy_elimination! {
    induction_variable: 0 -> 0;
    temporaries: a;
    labels: r#loop;
        (MOVE (TEMP a) (CONST 0))
        (JUMP r#loop)
    (LABEL r#loop)
        (MOVE (TEMP a) (ADD (TEMP a) (CONST 1)))
        (CJUMP (GE (TEMP a) (CONST 5)) r#loop)
}

partial_redundancy_elimination! {
    induction_variable_regression: 0 -> 0;
    temporaries: a, b, c;
    labels: r#while, r#true, r#false, black_box, exit;
        (MOVE (TEMP a) (_ARG 0))
        (MOVE (TEMP b) (CONST 0))
        (JUMP r#while)
    (LABEL r#while)
        (CJUMP (GE (TEMP b) (CONST 3)) r#true)
    (LABEL r#false)
        (CALL (NAME black_box) 1 (TEMP b))
        (MOVE (TEMP c) (_RET 0))
        (CALL (NAME black_box) 0 (TEMP c))
        (MOVE (TEMP b) (ADD (TEMP b) (CONST 1)))
        (JUMP r#while)
    (LABEL r#true)
        (RETURN)
        (JUMP exit)
}
