use io
use conv

class B extends A {
    x, y, z: int
    foo() {
        x = 1
        y = 2
        z = 3

        print("123 = ")
        print(unparseInt(x))
        print(unparseInt(y))
        println(unparseInt(z))

        super.foo()
    }
}

class A {
    z, x, y: int
    foo() {
        x = 3
        y = 2
        z = 1

        print("321 = ")
        print(unparseInt(x))
        print(unparseInt(y))
        println(unparseInt(z))
    }
}

main(args: int[][]) {
    b: B = new B
    b.foo()
}
