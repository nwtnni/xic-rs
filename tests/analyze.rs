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
    ($($instruction:tt)*) => {
        vec![$(asm!($instruction)),*]
    }
}

#[test]
fn basic() {
    let x = Temporary::Fixed(symbol::intern_static("x"));
    let y = Temporary::Fixed(symbol::intern_static("y"));
    let z = Temporary::Fixed(symbol::intern_static("z"));

    let enter = Label::Fixed(symbol::intern_static("enter"));
    let exit = Label::Fixed(symbol::intern_static("exit"));

    let instructions = assembly!(
        (enter:)
        (mov x, 1)
        (mov y, 2)
        (mov z, x)
        (add z, y)
        (mov z, 7)
        (mov x, z)
        (call<0, 0> x)
        (exit:)
        (ret<0>)
    );

    let function = Function {
        name: symbol::intern_static("test_mov_function"),
        instructions,
        arguments: 0,
        returns: 0,
        callee_arguments: 0,
        callee_returns: 0,
        enter,
        exit,
    };

    let cfg = xic::api::construct_cfg(&function);

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
