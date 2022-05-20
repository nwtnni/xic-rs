use io
use conv

main(args:int[][]) {
    a0: int = 0
    a1: int = 1
    a2: int = 2
    a3: int = 3
    a4: int = 4
    a5: int = 5
    a6: int = 6
    a7: int = 7
    a8: int = 8
    a9: int = 9
    a10: int = 10
    a11: int = 11
    a12: int = 12
    a13: int = 13
    a14: int = 14
    a15: int = 15
    a16: int = 16
    a17: int = 17
    a18: int = 18
    a19: int = 19
    a20: int = 20
    a21: int = 21
    a22: int = 22
    a23: int = 23
    a24: int = 24
    a25: int = 25
    a26: int = 26
    a27: int = 27
    a28: int = 28
    a29: int = 29

    b: int = ((((7 * 2) / 3) * 27) * 9999999989) *>> ((11 % 7) * 999999999999)
    c: int = b / 1 / 2 % 1000 % 100 * 50 % 10
    println(unparseInt(b + b - c))

    println(unparseInt(a29))
    println(unparseInt(a28))
    println(unparseInt(a27))
    println(unparseInt(a26))
    println(unparseInt(a25))
    println(unparseInt(a24))
    println(unparseInt(a23))
    println(unparseInt(a22))
    println(unparseInt(a21))
    println(unparseInt(a20))
    println(unparseInt(a19))
    println(unparseInt(a18))
    println(unparseInt(a17))
    println(unparseInt(a16))
    println(unparseInt(a15))
    println(unparseInt(a14))
    println(unparseInt(a13))
    println(unparseInt(a12))
    println(unparseInt(a11))
    println(unparseInt(a10))
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
