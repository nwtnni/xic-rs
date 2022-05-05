use io
use conv

main(args:int[][]) {
    a0: int = 0
    recurse(5)
    a1: int = 1
    recurse(5)
    a2: int = 2
    recurse(5)
    a3: int = 3
    recurse(5)
    a4: int = 4
    recurse(5)
    a5: int = 5
    recurse(5)
    a6: int = 6
    recurse(5)
    a7: int = 7
    recurse(5)
    a8: int = 8
    recurse(5)
    a9: int = 9
    recurse(5)

    println(unparseInt(a9))
    println(unparseInt(a8))
    println(unparseInt(a7))
    println(unparseInt(a6))
    println(unparseInt(a5))
    println(unparseInt(a4))
    println(unparseInt(a3))
    println(unparseInt(a2))
    println(unparseInt(a1))
    println(unparseInt(a0))
}

recurse(i: int) {
    if (i == 0) {
        return
    } else {
        recurse(i - 1)
    }
}
