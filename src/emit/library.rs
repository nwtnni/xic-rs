use crate::abi;
use crate::data::hir;
use crate::data::ir;
use crate::data::operand::Label;
use crate::data::operand::Temporary;
use crate::data::symbol;
use crate::hir;

pub(super) fn emit_memdup() -> hir::Function {
    let address = Temporary::fresh("address");
    let offset = Temporary::fresh("offset");
    let bound = Temporary::fresh("bound");

    let r#while = Label::fresh("while");
    let done = Label::fresh("done");

    hir::Function {
        name: symbol::intern_static(abi::XI_MEMDUP),
        arguments: 1,
        returns: 1,
        linkage: ir::Linkage::LinkOnceOdr,
        statement: hir!(
            (SEQ
                (MOVE
                    (TEMP bound)
                    (ADD (MUL (MEM (TEMP Temporary::Argument(0))) (CONST abi::WORD)) (CONST abi::WORD)))
                (MOVE (TEMP address) (CALL (NAME abi::XI_ALLOC) 1 (TEMP bound)))
                (MOVE (TEMP offset) (CONST 0))
                (LABEL r#while)
                (MOVE
                    (MEM (ADD (TEMP address) (TEMP offset)))
                    (MEM (ADD (TEMP (Temporary::Argument(0))) (TEMP offset))))
                (MOVE (TEMP offset) (ADD (TEMP offset) (CONST abi::WORD)))
                (CJUMP (GE (TEMP offset) (TEMP bound)) done r#while)
                (LABEL done)
                (RETURN (TEMP address)))
        ),
    }
}

pub(super) fn emit_concat() -> hir::Function {
    let array_left = Temporary::fresh("array");
    let array_right = Temporary::fresh("array");
    let array = Temporary::fresh("array");

    let length_left = Temporary::fresh("length");
    let length_right = Temporary::fresh("length");
    let length = Temporary::fresh("length");

    let while_left = Label::fresh("while");
    let done_left = Label::fresh("true");
    let bound_left = Temporary::fresh("bound");

    let while_right = Label::fresh("while");
    let done_right = Label::fresh("true");
    let bound_right = Temporary::fresh("bound");

    let address = Temporary::fresh("address");

    hir::Function {
        name: symbol::intern_static(abi::XI_CONCAT),
        arguments: 2,
        returns: 1,
        linkage: ir::Linkage::LinkOnceOdr,
        statement: hir!(
            (SEQ
                (MOVE (TEMP array_left) (TEMP Temporary::Argument(0)))
                (MOVE (TEMP length_left) (MEM (SUB (TEMP array_left) (CONST abi::WORD))))

                (MOVE (TEMP array_right) (TEMP Temporary::Argument(1)))
                (MOVE (TEMP length_right) (MEM (SUB (TEMP array_right) (CONST abi::WORD))))

                // Allocate destination with correct length (+1 for length metadata)
                (MOVE (TEMP length) (ADD (TEMP length_left) (TEMP length_right)))
                (MOVE
                    (TEMP array)
                    (CALL (NAME abi::XI_ALLOC) 1 (ADD (MUL (TEMP length) (CONST abi::WORD)) (CONST abi::WORD))))
                (MOVE (MEM (TEMP array)) (TEMP length))
                (MOVE (TEMP address) (ADD (TEMP array) (CONST abi::WORD)))

                // Copy left array into final destination, starting at
                // `array + WORD`
                (MOVE (TEMP bound_left) (ADD (TEMP array_left) (MUL (TEMP length_left) (CONST abi::WORD))))
                (CJUMP (AE (TEMP array_left) (TEMP bound_left)) done_left while_left)
                (LABEL while_left)
                (MOVE (MEM (TEMP address)) (MEM (TEMP array_left)))
                (MOVE (TEMP array_left) (ADD (TEMP array_left) (CONST abi::WORD)))
                (MOVE (TEMP address) (ADD (TEMP address) (CONST abi::WORD)))
                (CJUMP (AE (TEMP array_left) (TEMP bound_left)) done_left while_left)
                (LABEL done_left)

                // Copy right array into final destination, starting at
                // `array + WORD + length_left * WORD`
                (MOVE (TEMP bound_right) (ADD (TEMP array_right) (MUL (TEMP length_right) (CONST abi::WORD))))
                (CJUMP (AE (TEMP array_right) (TEMP bound_right)) done_right while_right)
                (LABEL while_right)
                (MOVE (MEM (TEMP address)) (MEM (TEMP array_right)))
                (MOVE (TEMP array_right) (ADD (TEMP array_right) (CONST abi::WORD)))
                (MOVE (TEMP address) (ADD (TEMP address) (CONST abi::WORD)))
                (CJUMP (AE (TEMP array_right) (TEMP bound_right)) done_right while_right)
                (LABEL done_right)

                (RETURN (ADD (TEMP array) (CONST abi::WORD))))
        )
    }
}
