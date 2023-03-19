# xic-rs

An (extended) compiler for the Xi++ programming language, based on the language
specification and assignments from Cornell's [CS 4120 spring 2019 website](https://www.cs.cornell.edu/courses/cs4120/2019sp/).
The relevant documents are in the [`docs` directory](docs).

# Language Features

- Object-oriented
- Class inheritance and subtyping
- Dynamic dispatch
- Integer, array, and product (object) types
- C++ style duck-typed function and class templates
- C++ style header files
- Global static variables

# Compiler Features

- Six-ish intermediate representations
  - [Abstract syntax tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree) (AST):
    close to source language, preserves span information for error messages
  - High intermediate representation (HIR):
    desugared AST, arbitrarily nested trees of expressions and statements
  - Low intermediate representation (LIR):
    sequence of top-level statements with arbitrarily nested sub-expressions,
    generic over conditional branch (jump to label or fall through on false)
  - Abstract assembly:
    subset of x86-64, generic over temporary or register operands

- HIR and LIR interpreters

- (Dubious) macro-based domain-specific languages for LIR, HIR, assembly:

  ```rust
  hir::Function {
    name: symbol::intern_static(abi::XI_MEMDUP),
    linkage: ir::Linkage::LinkOnceOdr,
    statement: hir!(
        (SEQ
            (MOVE
                (TEMP bound)
                (ADD (MUL (MEM (TEMP arguments[0])) (CONST abi::WORD)) (CONST abi::WORD)))
            (MOVE (TEMP address) (CALL (NAME abi::XI_ALLOC) (Temporary::fresh_returns(1)) (TEMP bound)))
            (MOVE (TEMP offset) (CONST 0))
            (LABEL r#while)
            (MOVE
                (MEM (ADD (TEMP address) (TEMP offset)))
                (MEM (ADD (TEMP arguments[0]) (TEMP offset))))
            (MOVE (TEMP offset) (ADD (TEMP offset) (CONST abi::WORD)))
            (CJUMP (GE (TEMP offset) (TEMP bound)) done r#while)
            (LABEL done)
            (RETURN (TEMP address)))
    ),
    arguments,
    returns: 1,
  }
  ```

- Nice error messages via [ariadne](https://github.com/zesterer/ariadne):

  ```text
  >> xic tests/check/bad_incomplete_class_1.xi
  Error: Semantic error
    ╭─[./tests/check/bad_incomplete_class_1.xi:1:1]
    │
  1 │ class A {}
    · ─────┬────
    ·      ╰────── Class A does not implement method required in interface
    │
    ├─[./tests/check/bad_incomplete_class_1.ixi:2:5]
    │
  2 │     foo(): int
    ·     ─┬─
    ·      ╰─── Method required here
  ───╯
  ```

- Correctness
  - Suite of ~5000 snapshot tests via [insta](https://insta.rs/)
  - UI testing of lexer, parser, type checker errors
  - Manual testing of [Qt examples](runtime-qt/examples)
  - Property testing of behavior equivalence across optimizations and interpreters
  - Working [solutions](tests/advent) for some [Advent of Code](http://adventofcode.com/) problems

- Optimization
  - Generic [dataflow analysis](https://en.wikipedia.org/wiki/Data-flow_analysis) framework
  - [Loop inversion](https://en.wikipedia.org/wiki/Loop_inversion)
  - Static dispatch for final classes
  - [Constant folding](https://en.wikipedia.org/wiki/Constant_folding)
  - [Function inlining](https://en.wikipedia.org/wiki/Inline_expansion)
  - [Copy propagation](https://en.wikipedia.org/wiki/Copy_propagation)
  - [Conditional constant propagation](https://www.cs.cornell.edu/courses/cs4120/2020sp/lectures/23ccp/lec23-sp19.pdf)
  - [Constant propagation](https://en.wikipedia.org/wiki/Constant_folding#Constant_propagation)
  - [Dead code elimination](https://en.wikipedia.org/wiki/Dead-code_elimination)
  - [Partial redundancy elimination](https://en.wikipedia.org/wiki/Partial-redundancy_elimination)
  - [Frame pointer omission](https://stackoverflow.com/questions/14666665/trying-to-understand-gcc-option-fomit-frame-pointer)
  - [Linear scan register allocation](http://web.cs.ucla.edu/~palsberg/course/cs132/linearscan.pdf)

- Debugging
  - Optimization effects logged via [pretty_env_logger](https://docs.rs/pretty_env_logger/latest/pretty_env_logger/)
  - [graphviz](https://graphviz.org/)-based rendering of dataflow solutions:

    ```text
    >> cat tests/suite/analyze/snapshots/suite__analyze__copy_propagation__partially_overwritten_tree.snap
    ---
    source: tests/suite/analyze/copy_propagation.rs
    assertion_line: 90
    expression: copy_propagation(function)
    ---

        partially_overwritten_tree

    +----------------------------------+
    |              enter:              |
    | {}                               |
    | mov _a, _b                       |
    | {_a: _b}                         |
    | mov _d, _a                       |
    | {_a: _b, _d: _a}                 |
    | mov _c, _b                       |
    | {_a: _b, _d: _a, _c: _b}         |
    | mov _e, _c                       |
    | {_a: _b, _d: _a, _c: _b, _e: _c} |
    | mov _a, 1                        |
    | {_e: _c, _c: _b}                 |
    | jmp exit                         |
    | {_e: _c, _c: _b}                 |
    +----------------------------------+
      |
      |
      v
    +----------------------------------+
    |              exit:               |
    | {_e: _c, _c: _b}                 |
    | ret                              |
    | {_e: _c, _c: _b}                 |
    +----------------------------------+
    ```
  - Custom rendering of live variable ranges:

    ```text
    >> cat tests/suite/analyze/snapshots/suite__analyze__live_variables__everything_is_meaningless_except_call-2.snap
    ---
    source: tests/suite/analyze/live_variables.rs
    assertion_line: 198
    expression: live_ranges
    ---
    rsp          enter:
    ┊              mov _x, 1
    ┊   _x         mov _y, 2
    ┊   |   _y     add _x, _y
    ┊   |   |      add _y, _x
    ┊   ●   |      mov rdi, _x
    ┊   rdi ●      mov rsi, _y
    ┊   ●   rsi    call black_box
    ┊              add _x, _y
    ┊              add _y, _x
    ┊            exit:
    C              ret
    ```
