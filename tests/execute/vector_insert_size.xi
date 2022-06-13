use assert
use io
use conv
use vector

main(args: int[][]) {

    output: Vector::<int> = newVector::<int>()

    i: int = 0
    while i < 100 {
        output.insert(output.size(), i)
        assert(output.size() == i + 1)
        i = i + 1
    }

    print("100 = ")
    println(unparseInt(output.size()))
}
