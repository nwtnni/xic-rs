use io
use conv

template class A<T> {
    field: T
}

main(args: int[][]) {
    a: A::<A::<int>> = new A::<A::<int>>
    a.field = new A::<int>
    a.field.field = 1
    print("1 = ")
    println(unparseInt(a.field.field))
}
