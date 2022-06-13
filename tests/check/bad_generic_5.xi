class B {
    bar(): int {
        return 1
    }
}

main(args: int[][]) {
    a: A::<B> = new A::<B>
    b: A::<int> = new A::<int>
}
