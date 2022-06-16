use io

class A {
    foo(): int[] {
        return "in class A: foo"
    }
}

final class B extends A {
    foo(): int[] {
        return "in class B: foo"
    }

    bar(): int[] {
        return "in class B: bar"
    }
}

main(args: int[][]) {
    a: A = new A

    println(a.foo())

    a = new B

    println(a.foo())

    println(new B.bar())
}
