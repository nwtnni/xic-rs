template class A<T> {
    a: T
    foo() {
        a.b = 1
    }
}

class B {
    b: bool
}

main(args: int[][]) {
    a: A::<B> = new A::<B>
}
