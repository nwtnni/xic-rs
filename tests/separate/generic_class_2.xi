use assert
use generic_class_1
use io
use vector

main(args: int[][]) {
    vector: Vector::<int> = foo()

    j: int = 15
    while j >= 0 {
        assert(vector.pop() == j)
        j = j - 1
    }

    println("Did not crash!")
}
