template class A<T> {
    a: T
    foo() {
        a.b = 1
    }
}

main(args: int[][]) {
    a: A::<int> = new A::<int>
}
