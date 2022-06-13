template foo<T>(argument: T): T {
    return argument + argument
}

main(args: int[][]) {
    i: int = foo::<int>(1);
    b: bool = foo::<bool>(true);
}
