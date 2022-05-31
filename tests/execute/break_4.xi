use io
use conv

class A {
    i: int
    foo() {
        i = 0
        while i < 10 {
            println(unparseInt(i))
            if i > 5 {
                break
            }
            i = i + 1
        }
    }
}

main(args: int[][]) {
    a: A = new A
    a.foo()
}
