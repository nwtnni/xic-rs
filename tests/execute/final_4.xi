use io

final class B extends A::<> {
    bar(): int[] {
        return "in class B"
    }
}

main(args: int[][]) {
    a: A::<> = new A::<>

    println(a.foo())

    a = new B

    println(a.foo())

    println(new B.bar())
}
