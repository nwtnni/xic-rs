use io
use conv
use vector

template class A<T> {
    field: T
}

main(args: int[][]) {
    vector: Vector::<A::<int>> = new_vector::<A::<int>>()

    i: int = 0
    while i < 16 {
        a: A::<int> = new A::<int>
        a.field = i
        vector.push(a)
        i = i + 1
    }

    i = 0
    while i < 16 {
        print(unparseInt(vector.size()))
        print(": ")
        println(unparseInt(vector.pop().field))
        i = i + 1
    }
}
