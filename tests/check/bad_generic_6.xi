template class B<T> {
    field: T
    foo(): T {
        return field + field
    }
}

main(args: int[][]) {
    a: B::<int> = new B::<int>
    a.field = 1

    b: B::<bool> = new B::<bool>
}
