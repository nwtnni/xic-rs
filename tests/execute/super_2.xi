use io
use conv

class B extends A {
    foo(): A {
        super.x = 1
        super.y = 2
        return super
    }
}

class A {
    x, y: int

    bar() {
        print("3 = ")
        println(unparseInt(x + y))
    }
}

main(args: int[][]) {
    b: B = new B
    b.foo().bar()
}

