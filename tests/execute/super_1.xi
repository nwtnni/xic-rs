use io
use conv

class B extends A {
    x: int

    superGet(): int {
        return super.x
    }

    superSet(x: int) {
        super.x = x
    }

    get(): int {
        return x
    }

    set(x: int) {
        this.x = x
    }
}

class A {
    x: int
}

main(args: int[][]) {
    b: B = new B

    b.set(1)
    b.superSet(2)
    println(unparseInt(b.get()))
    println(unparseInt(b.superGet()))
}
