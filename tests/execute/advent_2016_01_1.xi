use assert
use io
use conv

input: int[] = "L5, R1, R3, L4, R3, R1, L3, L2, R3, L5, L1, L2, R5, L1, R5, R1, L4, R1, R3, L4, L1, R2, R5, R3, R1, R1, L1, R1, L1, L2, L1, R2, L5, L188, L4, R1, R4, L3, R47, R1, L1, R77, R5, L2, R1, L2, R4, L5, L1, R3, R187, L4, L3, L3, R2, L3, L5, L4, L4, R1, R5, L4, L3, L3, L3, L2, L5, R1, L2, R5, L3, L4, R4, L5, R3, R4, L2, L1, L4, R1, L3, R1, R3, L2, R1, R4, R5, L3, R5, R3, L3, R4, L2, L5, L1, L1, R3, R1, L4, R3, R3, L2, R5, R4, R1, R3, L4, R3, R3, L2, L4, L5, R1, L4, L5, R4, L2, L1, L3, L3, L5, R3, L4, L3, R5, R4, R2, L4, R2, R3, L3, R4, L1, L3, R2, R1, R5, L4, L5, L5, R4, L5, L2, L4, R4, R4, R1, L3, L2, L4, R3"

L: int = input[0]
R: int = input[4]
COMMA: int = input[2]

main(args: int[][]) {

    i: int = 0

    //     N 0
    // W 3     E 1
    //     S 2
    direction: int = 0

    x: int = 0
    y: int = 0

    while i < length(input) {

        lo: int = i + 1
        hi: int = i + 1

        // Parse block distance
        do {
            if input[hi] == COMMA {
                break
            }
            hi = hi + 1
        } while hi < length(input)

        slice: int[hi - lo]

        j: int = 0
        while j < length(slice) {
            slice[j] = input[lo + j]
            j = j + 1
        }

        blocks: int, valid: bool = parseInt(slice)
        assert(valid)

        // Parse direction
        if input[i] == L {
            direction = (direction + 3) % 4
        } else if input[i] == R {
            direction = (direction + 1) % 4
        } else {
            assert(false)
        }

        // Travel in direction
        if direction == 0 {
            y = y + blocks
        } else if direction == 1 {
            x = x + blocks
        } else if direction == 2 {
            y = y - blocks
        } else if direction == 3 {
            x = x - blocks
        } else {
            assert(false)
        }

        // Skip over comma, space
        i = hi + 2
    }

    println(unparseInt(abs(x) + abs(y)))
}

abs(x: int): int {
    if x >= 0 {
        return x
    } else {
        return -x
    }
}
