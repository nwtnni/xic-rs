use io
use conv

a: A = new A.init()

class A {
    x: int

    init(): A {
        x = 1
        return this
    }
}

main(args: int[][]) {
    print("1 = ")
    println(unparseInt(a.x))
}
