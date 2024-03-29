use std::cmp;

use crate::analyze;
use crate::analyze::LiveRanges;
use crate::analyze::Range;
use crate::data::operand::Register;
use crate::data::operand::Temporary;
use crate::Map;

/// Requires that dead code elimination has been run on `ranges.function` using
/// the same live variable analysis.
pub fn allocate(
    ranges: &analyze::LiveRanges,
    registers: Vec<Register>,
) -> (Map<Temporary, Register>, Map<Temporary, usize>) {
    let mut linear = Linear::new(registers);
    linear.allocate(ranges);
    (linear.allocated, linear.spilled)
}

/// This register allocator is based on the research paper
/// "Linear scan register allocation" by Poletto and Sarkar:
///
/// https://dl.acm.org/doi/10.1145/330249.330250
///
/// There were some challenges when modifying the algorithm
/// (and live variable analysis) to work with the x86-64 ISA:
///
/// - Representing temporaries bound to fixed registers
///   (e.g. passing arguments, `imul` operands)
///
/// This is handled by "allocating" all fixed registers to
/// themselves. If the register is in use by some existing
/// temporary, then we spill the temporary to the stack.
///
/// It should be possible to select a better register when
/// first allocating a temporary by checking for range
/// intersections against fixed registers, but this is
/// not implemented.
///
/// - Preserving caller-saved registers across calls
///
/// This is handled by having live range analysis record whether
/// a live range crosses a function call. Any temporary whose
/// live range crosses the call cannot use a caller-saved register.
///
/// Handling clobbering from `imul` and co. uses the same workaround.
///
/// - Maintaining valid addressing modes when spilling
///   temporaries to the stack
///
/// This I'm punting on for now, by conservatively reserving
/// two dedicated spill registers.
struct Linear {
    active: Vec<cmp::Reverse<(usize, Temporary)>>,
    allocated: Map<Temporary, Register>,
    spilled: Map<Temporary, usize>,
    registers: Vec<Register>,
}

impl Linear {
    fn new(registers: Vec<Register>) -> Self {
        Linear {
            active: Vec::new(),
            allocated: Map::default(),
            spilled: Map::default(),
            registers,
        }
    }

    fn allocate(&mut self, ranges: &LiveRanges) {
        let mut ranges = ranges
            .ranges
            .iter()
            .map(|(temporary, range)| (*range, *temporary))
            .collect::<Vec<_>>();

        ranges.sort();

        for (range, temporary) in ranges {
            self.expire(range.start);
            self.allocate_temporary(temporary, range);
        }
    }

    fn allocate_temporary(&mut self, temporary: Temporary, range: Range) {
        if let Temporary::Register(register) = temporary {
            self.allocate_register(register, range);
            return;
        }

        let register = self
            .registers
            .iter()
            .rposition(|register| !range.clobbered.as_slice().contains(register))
            .map(|index| self.registers.remove(index));

        if let Some(register) = register {
            self.allocated.insert(temporary, register);
            self.active.push(cmp::Reverse((range.end, temporary)));
            self.active.sort();
            return;
        }

        // Find latest ending temporary, skipping over fixed and clobbered registers
        match self
            .active
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, cmp::Reverse((_, temporary)))| {
                !range
                    .clobbered
                    .as_slice()
                    .contains(&self.allocated[temporary])
            })
            .find(|(_, cmp::Reverse((_, temporary)))| !matches!(temporary, Temporary::Register(_)))
        {
            Some((index, cmp::Reverse((end, existing)))) if end > range.end => {
                let register = self.allocated[&existing];

                self.spill(existing);

                self.allocated.insert(temporary, register);
                self.active[index] = cmp::Reverse((range.end, temporary));
                self.active.sort();
            }
            Some(_) | None => self.spill(temporary),
        }
    }

    fn allocate_register(&mut self, register: Register, range: Range) {
        if !self.registers.contains(&register) {
            // Find active temporary using the register: there must be exactly one!
            let (index, &cmp::Reverse((_, temporary))) = self
                .active
                .iter()
                .enumerate()
                .find(|(_, cmp::Reverse((_, temporary)))| self.allocated[temporary] == register)
                .unwrap();

            self.spill(temporary);
            self.active.remove(index);
        }

        self.registers.retain(|available| *available != register);
        self.allocated
            .insert(Temporary::Register(register), register);
        self.active
            .push(cmp::Reverse((range.end, Temporary::Register(register))));
        self.active.sort();
    }

    fn expire(&mut self, start: usize) {
        while let Some(cmp::Reverse((end, temporary))) = self.active.last() {
            // With our implementation of live ranges, using `>=` here is
            // important. Consider the following sequence of statements,
            // with live ranges annotated on the right:
            //
            // ```text
            //     .            |
            //     .            |    |
            //     .            |    |
            // mov t1, t2       t2   |
            // mov t3, t4   t1       t4
            //     .        |   t3
            //     .        |   |
            //     .        |   |
            //     .            |
            // ```
            //
            // Here, `t1` and `t4` should not be assigned the same register,
            // or else `t3` will end up with the value of `t2`.
            if *end >= start {
                return;
            }

            let register = self.allocated[temporary];

            // Return to the free pool
            self.registers.push(register);
            self.active.pop();
        }
    }

    fn spill(&mut self, temporary: Temporary) {
        let index = self.spilled.len();
        self.allocated.remove(&temporary);
        self.spilled.insert(temporary, index);
    }
}
