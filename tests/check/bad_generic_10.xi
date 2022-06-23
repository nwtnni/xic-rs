template foo<T>() {}

bar() {
    foo::<bool, int>()
}
