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

macro_rules! lazy_code_motion {
    ($function:ident $($tt:tt)*) => {
        #[test]
        fn $function() {
            let function = lir_function!($function $($tt)*);
            let (anticipated, available, earliest, postponable, latest, used) = lazy_code_motion(function);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_anticipated"), anticipated);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_available"), available);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_earliest"), earliest);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_postponable"), postponable);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_latest"), latest);
            insta::assert_display_snapshot!(concat!(stringify!($function), "_used"), used);
        }
    }
}

fn lazy_code_motion(
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

lazy_code_motion! {
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

lazy_code_motion! {
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
