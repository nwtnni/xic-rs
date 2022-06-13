template class B<T> {
    field: T
    foo(): T {
        return field + field
    }
}

main(args: int[][]) {
    a: A::<B, int> = new A::<B, int>
    a.a.field = 1

    b: A::<B, bool> = new A::<B, bool>
}
