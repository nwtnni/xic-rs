use assert
use conv
use io
use vector

newIntVector(): Vector::<int> {
    return newVector::<int>()
}

newBoolVector(): Vector::<bool> {
    return newVector::<bool>()
}

main(args: int[][]) {
    ints: Vector::<int> = newIntVector()
    bools: Vector::<bool> = newBoolVector()

    i: int = 0
    while i < 128 {
        ints.push(i)
        bools.push(i % 2 == 0)
        i = i + 1
    }

    j: int = ints.size()
    while j > 0 {
        if j % 2 == 0 {
            assert((ints.remove(0) % 2 == 0) == bools.remove(0))
        } else {
            assert((ints.swap_remove(0) % 2 == 0) == bools.swap_remove(0))
        }

        j = j - 1
    }

    println("Did not crash!")
}
