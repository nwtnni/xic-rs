use xic::api::analyze::analyze;
use xic::api::analyze::display;
use xic::api::analyze::AnticipatedExpressions;
use xic::data::lir::Fallthrough;
use xic::data::lir::Function;

macro_rules! anticipated_expressions {
    ($function:ident $($tt:tt)*) => {
        #[test]
        fn $function() {
            let function = lir_function!($function $($tt)*);
            insta::assert_display_snapshot!(anticipated_expressions(function));
        }
    }
}

fn anticipated_expressions(function: Function<Fallthrough>) -> String {
    let cfg = xic::api::construct_cfg(function);
    let live_variables = analyze::<AnticipatedExpressions, _>(&cfg);
    super::super::graph(display(&live_variables, &cfg))
}

anticipated_expressions! {
    used_on_both_branches: 0 -> 0;
    temporaries: a, b, c, d;
    labels: branch, fallthrough, exit;
    (MOVE (TEMP a) (ADD (TEMP b) (CONST 1)))
    (CJUMP (GE (TEMP a) (TEMP b)) branch)
    (LABEL fallthrough)
    (MOVE (TEMP c) (ADD (TEMP b) (CONST 1)))
    (JUMP exit)
    (LABEL branch)
    (MOVE (TEMP d) (ADD (TEMP b) (CONST 1)))
    (JUMP exit)
}

anticipated_expressions! {
    used_on_one_branch: 0 -> 0;
    temporaries: a, b, c, d;
    labels: branch, fallthrough, exit;
    (MOVE (TEMP a) (ADD (TEMP b) (CONST 1)))
    (CJUMP (GE (TEMP a) (TEMP b)) branch)
    (LABEL fallthrough)
    (MOVE (TEMP c) (ADD (TEMP b) (CONST 1)))
    (JUMP exit)
    (LABEL branch)
    (MOVE (TEMP d) (ADD (TEMP b) (CONST 2)))
    (JUMP exit)
}
