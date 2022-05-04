use std::collections::BTreeMap;
use std::io::Write as _;
use std::process;

use xic::api::analyze::display;
use xic::api::analyze::LiveVariables;
use xic::asm;
use xic::data::asm::Function;
use xic::data::asm::Unit;
use xic::data::operand::Temporary;
use xic::data::symbol;

macro_rules! assembly {
    ($($instruction:tt)*) => {
        vec![$(asm!($instruction)),*]
    }
}

#[test]
fn basic() {
    let x = Temporary::Fixed(symbol::intern_static("x"));
    let y = Temporary::Fixed(symbol::intern_static("y"));
    let z = Temporary::Fixed(symbol::intern_static("z"));

    let instructions = assembly!(
        (mov x, 1)
        (mov y, 2)
        (mov z, x)
        (add z, y)
        (mov z, 7)
        (mov x, z)
        (call<0, 0> x)
    );

    let function = Function {
        name: symbol::intern_static("test_mov_function"),
        instructions,
        arguments: 0,
        returns: 0,
        callee_arguments: 0,
        callee_returns: 0,
    };

    let mut functions = BTreeMap::new();
    functions.insert(function.name, function);

    let unit = Unit {
        name: symbol::intern_static("test_mov_unit"),
        functions,
        data: BTreeMap::default(),
    };

    let cfg = xic::api::construct_cfg(&unit);

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

    let output = graph.wait_with_output().unwrap();

    if !output.status.success() {
        panic!("Failed to generate diagram from .dot file");
    }

    insta::assert_display_snapshot!(String::from_utf8(output.stdout).unwrap())
}
