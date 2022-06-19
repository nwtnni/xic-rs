use assert
use conv
use io
use math
use string
use vector_set

INPUT: int[] = "L5, R1, R3, L4, R3, R1, L3, L2, R3, L5, L1, L2, R5, L1, R5, R1, L4, R1, R3, L4, L1, R2, R5, R3, R1, R1, L1, R1, L1, L2, L1, R2, L5, L188, L4, R1, R4, L3, R47, R1, L1, R77, R5, L2, R1, L2, R4, L5, L1, R3, R187, L4, L3, L3, R2, L3, L5, L4, L4, R1, R5, L4, L3, L3, L3, L2, L5, R1, L2, R5, L3, L4, R4, L5, R3, R4, L2, L1, L4, R1, L3, R1, R3, L2, R1, R4, R5, L3, R5, R3, L3, R4, L2, L5, L1, L1, R3, R1, L4, R3, R3, L2, R5, R4, R1, R3, L4, R3, R3, L2, L4, L5, R1, L4, L5, R4, L2, L1, L3, L3, L5, R3, L4, L3, R5, R4, R2, L4, R2, R3, L3, R4, L1, L3, R2, R1, R5, L4, L5, L5, R4, L5, L2, L4, R4, R4, R1, L3, L2, L4, R3"

L: int = INPUT[0]
R: int = INPUT[4]
COMMA: int = INPUT[2]

main(args: int[][]) {
    input: String = new_string_from_array(INPUT)

    one: Done = new Done

    x: int, y: int = run(input, one)

    println(unparseInt(abs::<>(x) + abs::<>(y)))

    two: Done = new Done'.init()

    x': int, y': int = run(input, two)

    println(unparseInt(abs::<>(x') + abs::<>(y')))
}

run(input: String, done: Done): int, int {

    i: int = 0

    //     N 0
    // W 3     E 1
    //     S 2
    direction: int = 0

    x: int = 0
    y: int = 0

    while i < input.size() {

        low: int = i + 1
        high: int = i + 1

        // Parse block distance
        do {
            if input.get(high) == COMMA {
                break
            }
            high = high + 1
        } while high < input.size()

        blocks: int, valid: bool = parseInt(input.slice_array(low, high))
        assert(valid)

        // Parse direction
        if input.get(i) == L {
            direction = (direction + 3) % 4
        } else if input.get(i) == R {
            direction = (direction + 1) % 4
        } else {
            assert(false)
        }

        block: int = 0

        while block < blocks {
            // Travel in direction
            if direction == 0 {
                y = y + 1
            } else if direction == 1 {
                x = x + 1
            } else if direction == 2 {
                y = y - 1
            } else if direction == 3 {
                x = x - 1
            } else {
                assert(false)
            }

            if done.done(new Point.init(x, y)) {
                return x, y
            }

            block = block + 1
        }

        // Skip over comma, space
        i = high + 2
    }

    return x, y
}

class Done {
    done(point: Point): bool {
        return false
    }
}

final class Done' extends Done {
    visited: VectorSet::<Point>

    done(point: Point): bool {
        return !visited.insert(point)
    }

    init(): Done' {
        this.visited = new_vector_set::<Point>()
        return this
    }
}

final class Point {
    x, y: int

    equals(other: Point): bool {
        return this.x == other.x & this.y == other.y
    }

    init(x: int, y: int): Point {
        this.x = x
        this.y = y
        return this
    }
}
