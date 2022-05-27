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

        super.x = 2
        super.y = 1
        super.z = 3
        print("213 = ")
        print(unparseInt(super.x))
        print(unparseInt(super.y))
        println(unparseInt(super.z))

        this.z = 1
        this.y = 3
        this.x = 2
        print("132 = ")
        print(unparseInt(this.z))
        print(unparseInt(this.y))
        println(unparseInt(this.x))

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
