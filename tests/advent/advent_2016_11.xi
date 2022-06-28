use assert
use conv
use io
use math
use sort
use string
use vector
use vector_set
use vector_queue

// Edit to print the entire solution
DEBUG: bool = false
FLOORS: int = 4

// Note: microchips are encoded as 2, generators as 1

// The first floor contains a hydrogen-compatible microchip and a lithium-compatible microchip.
// The second floor contains a hydrogen generator.
// The third floor contains a lithium generator.
// The fourth floor contains nothing relevant.
EXAMPLE: State = new State.init(0, {
    {2, 2},
    {1, 0},
    {0, 1},
    {0, 0}
})

// The first floor contains a thulium generator, a thulium-compatible microchip, a plutonium generator, and a strontium generator.
// The second floor contains a plutonium-compatible microchip and a strontium-compatible microchip.
// The third floor contains a promethium generator, a promethium-compatible microchip, a ruthenium generator, and a ruthenium-compatible microchip.
// The fourth floor contains nothing relevant.
ONE: State = new State.init(0, {
    {3, 1, 1, 0, 0},
    {0, 2, 2, 0, 0},
    {0, 0, 0, 3, 3},
    {0, 0, 0, 0, 0}
})

// Upon entering the isolated containment area, however, you notice some extra parts on the first floor that weren't listed on the record outside:
// -  An elerium generator.
// -  An elerium-compatible microchip.
// -  A dilithium generator.
// -  A dilithium-compatible microchip.
TWO: State = new State.init(0, {
    {3, 1, 1, 0, 0, 3, 3},
    {0, 2, 2, 0, 0, 0, 0},
    {0, 0, 0, 3, 3, 0, 0},
    {0, 0, 0, 0, 0, 0, 0}
})

main(args: int[][]) {
    println(unparseInt(solve(EXAMPLE)))
    println(unparseInt(solve(ONE)))
    println(unparseInt(solve(TWO)))
}

solve(initial: State): int {
    queue: VectorQueue::<Node> = new_vector_queue::<Node>()
    queue.push(new Node.init(null, initial, 0))

    visited: VectorSet::<State> = new_vector_set::<State>()
    _ = visited.insert(queue.peek().state)

    deltas: Vector::<int[]> = new_vector::<int[]>()

    while queue.size() > 0 {
        node: Node = queue.pop();

        if node.state.done() {
            if DEBUG {
                node.debug()
            }
            return node.depth
        }

        node.state.generate(deltas)

        while deltas.size() > 0 {
            delta: int[] = deltas.pop()

            state: State = node.state.move(1, delta)
            state': State = node.state.move(-1, delta)

            if state != null & visited.insert(state) {
                queue.push(new Node.init(node, state, node.depth + 1))
            }

            if state' != null & visited.insert(state') {
                queue.push(new Node.init(node, state', node.depth + 1))
            }
        }
    }

    assert(false)
    return -1
}

final class Node {
    previous: Node
    state: State
    depth: int

    init(previous: Node, state: State, depth: int): Node {
        this.previous = previous
        this.state = state
        this.depth = depth
        return this
    }

    debug() {
        if previous != null {
            previous.debug()
        }

        println(unparseInt(depth))
        state.debug()
    }
}

final class State {
    elevator: int
    floors: int[][]
    hash': int

    init(elevator: int, floors: int[][]): State {
        this.elevator = elevator
        this.floors = floors
        this.hash' = hash()
        return this
    }

    done(): bool {
        i: int = 0
        floor: int[] = floors[FLOORS - 1]
        while i < length(floor) {
            if floor[i] < 3 {
                return false
            }
            i = i + 1
        }
        return true
    }

    move(direction: int, delta: int[]): State {
        elevator': int = elevator + direction

        if elevator' < 0 | elevator' >= FLOORS {
            return null
        }

        state': State = clone()
        state'.elevator = elevator'

        floor: int[] = state'.floors[elevator]
        floor': int[] = state'.floors[elevator']

        i: int = 0
        while i < length(floor) {
            floor[i] = floor[i] - delta[i]
            floor'[i] = floor'[i] + delta[i]
            i = i + 1
        }

        if state'.valid() {
            state'.hash' = state'.hash()
            return state'
        } else {
            return null
        }
    }

    generate(deltas: Vector::<int[]>) {
        floor: int[] = floors[elevator]
        elements: int = length(floor)

        i: int = 0
        while i < elements {
            // Try moving both
            if floor[i] == 3 {
                deltas.push(new_delta(elements, i, 3))
            }

            // Microchip available
            if floor[i] > 1 {
                delta: int[] = new_delta(elements, i, 2)
                deltas.push(delta)

                j: int = i + 1
                while j < length(floor) {
                    // Pair with other microchip
                    if floor[j] > 1 {
                        delta': int[] = clone_delta(delta)
                        delta'[j] = 2
                        deltas.push(delta')
                    }

                    j = j + 1
                }
            }

            // Generator available
            if floor[i] % 2 == 1 {
                delta: int[] = new_delta(elements, i, 1)
                deltas.push(delta)

                j: int = i + 1
                while j < length(floor) {
                    // Pair with other generator
                    if floor[j] % 2 == 1 {
                        delta': int[] = clone_delta(delta)
                        delta'[j] = 1
                        deltas.push(delta')
                    }

                    j = j + 1
                }
            }

            i = i + 1
        }
    }

    valid(): bool {
        i: int = 0
        while i < FLOORS {
            floor: int[] = floors[i]
            microchip: bool = false
            generator: bool = false

            j: int = 0

            while j < length(floor) {
                element: int = floor[j]

                has_generator: bool = element % 2 == 1
                has_microchip: bool = element == 2

                if microchip & has_generator | generator & has_microchip {
                    return false
                }

                generator = generator | has_generator
                microchip = microchip | has_microchip
                j = j + 1
            }

            i = i + 1
        }

        return true
    }

    equals(state: State): bool {
        return elevator == state.elevator & hash' == state.hash'
    }

    hash(): int {
        elements: int = length(floors[0])
        hash: Integer[elements]

        i: int = 0
        while i < elements {
            j: int = 0

            hash[i] = new Integer.init(0)

            while j < FLOORS {
                hash[i].value = hash[i].value + pow::<>(4, j) * floors[j][i]
                j = j + 1
            }

            i = i + 1
        }

        // Exploit symmetry
        bubble_sort_array::<Integer>(hash)

        sum: int = 0
        k: int = 0
        while k < elements {
            sum = sum + pow::<>(pow::<>(4, FLOORS), k) * hash[k].value
            k = k + 1
        }

        return sum
    }

    clone(): State {
        elements: int = length(floors[0])
        floors: int[FLOORS][elements]

        i: int = 0
        while i < FLOORS {
            floor: int[elements]

            j: int = 0
            while j < length(floor) {
                floor[j] = this.floors[i][j]
                j = j + 1
            }

            floors[i] = floor
            i = i + 1
        }

        return new State.init(elevator, floors)
    }

    debug() {
        i: int = FLOORS - 1
        while i >= 0 {
            if i == elevator {
                print("> ")
            } else {
                print("  ")
            }

            floor: int[] = floors[i]

            j: int = 0
            while j < length(floor) {
                print(unparseInt(floor[j]))
                print(" ")
                j = j + 1
            }

            println("")
            i = i - 1
        }
    }
}

final class Integer {
    value: int

    init(value: int): Integer {
        this.value = value
        return this
    }

    compare(integer: Integer): int {
        return value - integer.value
    }
}

new_delta(elements: int, element: int, value: int): int[] {
    delta: int[elements]

    i: int = 0
    while i < length(delta) {
        delta[i] = 0
        i = i + 1
    }

    delta[element] = value
    return delta
}

clone_delta(delta: int[]): int[] {
    delta': int[length(delta)]

    i: int = 0
    while i < length(delta') {
        delta'[i] = delta[i]
        i = i + 1
    }

    return delta'
}
