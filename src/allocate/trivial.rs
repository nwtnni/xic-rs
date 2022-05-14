use crate::data::asm;
use crate::data::operand;
use crate::data::operand::Memory;
use crate::data::operand::Temporary;
use crate::Map;

pub fn allocate(function: &asm::Function<Temporary>) -> Map<Temporary, usize> {
    let mut trivial = Trivial::default();
    for statement in &function.statements {
        trivial.allocate_statement(statement);
    }
    trivial.spilled
}

#[derive(Default)]
struct Trivial {
    spilled: Map<Temporary, usize>,
}

impl Trivial {
    fn allocate_statement(&mut self, statement: &asm::Statement<Temporary>) {
        match statement {
            asm::Statement::Nullary(_)
            | asm::Statement::Label(_)
            | asm::Statement::Jmp(_)
            | asm::Statement::Jcc(_, _) => {}
            asm::Statement::Binary(_, operands) => self.allocate_binary(operands),
            asm::Statement::Unary(_, operand) => self.allocate_unary(operand),
        }
    }

    fn allocate_binary(&mut self, binary: &operand::Binary<Temporary>) {
        self.allocate_unary(&operand::Unary::from(binary.destination()));
        self.allocate_unary(&binary.source());
    }

    fn allocate_unary(&mut self, unary: &operand::Unary<Temporary>) {
        match unary {
            operand::Unary::I(_) => (),
            operand::Unary::R(temporary) => self.allocate(temporary),
            operand::Unary::M(memory) => self.allocate_memory(memory),
        }
    }

    fn allocate_memory(&mut self, memory: &Memory<Temporary>) {
        memory.map(|temporary| self.allocate(temporary));
    }

    fn allocate(&mut self, temporary: &Temporary) {
        if let Temporary::Register(_) = temporary {
            return;
        }

        let index = self.spilled.len();
        self.spilled.entry(*temporary).or_insert(index);
    }
}
