template foo<T>(): T {
    return 1
}

template class A<T> {
    bar(): T {
        return foo::<T>()
    }
}

baz() {
    a: A::<int> = new A::<int>
    b: A::<bool> = new A::<bool>
}
