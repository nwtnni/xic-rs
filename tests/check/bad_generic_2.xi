template foo<T>(argument: T): T {
    return argument + argument
}

template bar<T>(argument: T): T {
    return foo::<T>(argument)
}

main(args: int[][]) {
    i: int = bar::<int>(1);
    b: bool = bar::<bool>(true);
}
