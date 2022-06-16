use io

final class A {
    foo(): int[] {
        return "in class A"
    }
}

main(args: int[][]) {
    println(new A.foo())
}
