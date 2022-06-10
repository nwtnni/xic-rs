use io
use conv

foo: int = 1

class A {
    foo: int
    bar(foo: int) {
        this.foo = 2
        print("2 = ")
        println(unparseInt(this.foo))
        print("3 = ")
        println(unparseInt(foo))
    }
}

main(args: int[][]) {
    a: A = new A
    print("1 = ")
    println(unparseInt(foo))
    a.bar(3)
}
