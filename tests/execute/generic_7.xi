use io
use conv
use vector

main(args: int[][]) {
    vector: Vector::<int> = newVector::<int>()

    i: int = 0
    while i < 16 {
        vector.push(i)
        i = i + 1
    }

    i = 0
    while i < 16 {
        print(unparseInt(vector.size()))
        print(": ")
        println(unparseInt(vector.pop()))
        i = i + 1
    }
}
