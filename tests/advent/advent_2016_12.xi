use assert
use conv
use io
use string
use vector

INPUT: String = new_string_from_array("\
    cpy 1 a\n\
    cpy 1 b\n\
    cpy 26 d\n\
    jnz c 2\n\
    jnz 1 5\n\
    cpy 7 c\n\
    inc d\n\
    dec c\n\
    jnz c -2\n\
    cpy a c\n\
    inc a\n\
    dec b\n\
    jnz b -2\n\
    cpy c b\n\
    dec d\n\
    jnz d -6\n\
    cpy 19 c\n\
    cpy 11 d\n\
    inc a\n\
    dec d\n\
    jnz d -2\n\
    dec c\n\
    jnz c -5")

CPY: String = new_string_from_array("cpy")
JNZ: String = new_string_from_array("jnz")
INC: String = new_string_from_array("inc")
DEC: String = new_string_from_array("dec")

main(args: int[][]) {
    instructions: Vector::<Instruction> = parse(INPUT.split_character('\n'))
    registers: int[4]

    registers[0] = 0
    registers[1] = 0
    registers[2] = 0
    registers[3] = 0
    println(unparseInt(run(instructions, registers)))

    registers[0] = 0
    registers[1] = 0
    registers[2] = 1
    registers[3] = 0
    println(unparseInt(run(instructions, registers)))
}

run(instructions: Vector::<Instruction>, registers: int[]): int {
    ip: int = 0

    while ip >= 0 & ip < instructions.size() {
        ip = instructions.get(ip).evaluate(ip, registers)
    }

    return registers[0]
}

parse(input: Vector::<String>): Vector::<Instruction> {
    instructions: Vector::<Instruction> = new_vector::<Instruction>()

    i: int = 0
    while i < input.size() {
        instructions.push(parse_instruction(input.get(i)))
        i = i + 1
    }

    return instructions
}

class Instruction {
    evaluate(ip: int, registers: int[]): int {
        return 0
    }
}

parse_instruction(instruction: String): Instruction {
    split: Vector::<String> = instruction.split_character(' ')

    if instruction.starts_with(CPY) {
        cpy: Cpy = new Cpy
        cpy.source = parse_register(split.get(1), true)
        cpy.destination = parse_register(split.get(2), true)
        return cpy
    } else if instruction.starts_with(JNZ) {
        jnz: Jnz = new Jnz
        jnz.register = parse_register(split.get(1), false)
        jnz.offset = parse_register(split.get(2), false)
        return jnz
    } else if instruction.starts_with(INC) {
        inc: Inc = new Inc
        inc.register = parse_register(split.get(1), false)
        return inc
    } else if instruction.starts_with(DEC) {
        dec: Dec = new Dec
        dec.register = parse_register(split.get(1), false)
        return dec
    } else {
        assert(false)
        return null
    }
}

// If `negate`, assumes `register` is either a register or a positive literal
parse_register(register: String, negate: bool): int {
    first: int = register.get(0)
    register': int

    if first == 'a' {
        register' = 0
    } else if first == 'b' {
        register' = 1
    } else if first == 'c' {
        register' = 2
    } else if first == 'd' {
        register' = 3
    } else {
        value: int, success: bool = parseInt(register.get_array())
        assert(success)
        if negate {
            register' = -value
        } else {
            register' = value
        }
    }

    return register'
}

final class Cpy extends Instruction {
    source: int
    destination: int

    evaluate(ip: int, registers: int[]): int {
        if source >= 0 {
            registers[destination] = registers[source]
        } else {
            registers[destination] = -source
        }

        return ip + 1
    }
}

final class Jnz extends Instruction {
    register: int
    offset: int

    evaluate(ip: int, registers: int[]): int {
        if registers[register] == 0 {
            return ip + 1
        } else {
            return ip + offset
        }
    }
}

final class Inc extends Instruction {
    register: int

    evaluate(ip: int, registers: int[]): int {
        registers[register] = registers[register] + 1
        return ip + 1
    }
}

final class Dec extends Instruction {
    register: int

    evaluate(ip: int, registers: int[]): int {
        registers[register] = registers[register] - 1
        return ip + 1
    }
}
