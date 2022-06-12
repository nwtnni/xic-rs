use io
use conv

template class Outer<T> {
    inner: T
}

main(args: int[][]) {
    a: Outer::<int> = new Outer::<int>
    b: Outer::<bool> = new Outer::<bool>

    a.inner = 1
    b.inner = true

    if b.inner {
        println(unparseInt(a.inner))
    }
}
