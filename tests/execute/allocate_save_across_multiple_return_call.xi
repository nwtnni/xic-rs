use io
use conv

main(args:int[][]) {
    a0: int, _, _, _, _, _, _, _, _, _ = pressure()
    _, a1: int, _, _, _, _, _, _, _, _ = pressure()
    _, _, a2: int, _, _, _, _, _, _, _ = pressure()
    _, _, _, a3: int, _, _, _, _, _, _ = pressure()
    _, _, _, _, a4: int, _, _, _, _, _ = pressure()
    _, _, _, _, _, a5: int, _, _, _, _ = pressure()
    _, _, _, _, _, _, a6: int, _, _, _ = pressure()
    _, _, _, _, _, _, _, a7: int, _, _ = pressure()
    _, _, _, _, _, _, _, _, a8: int, _ = pressure()
    _, _, _, _, _, _, _, _, _, a9: int = pressure()

    a10: int, _, _, _, _, _, _, _, _, _ = pressure()
    _, a11: int, _, _, _, _, _, _, _, _ = pressure()
    _, _, a12: int, _, _, _, _, _, _, _ = pressure()
    _, _, _, a13: int, _, _, _, _, _, _ = pressure()
    _, _, _, _, a14: int, _, _, _, _, _ = pressure()
    _, _, _, _, _, a15: int, _, _, _, _ = pressure()
    _, _, _, _, _, _, a16: int, _, _, _ = pressure()
    _, _, _, _, _, _, _, a17: int, _, _ = pressure()
    _, _, _, _, _, _, _, _, a18: int, _ = pressure()
    _, _, _, _, _, _, _, _, _, a19: int = pressure()

    a20: int, _, _, _, _, _, _, _, _, _ = pressure()
    _, a21: int, _, _, _, _, _, _, _, _ = pressure()
    _, _, a22: int, _, _, _, _, _, _, _ = pressure()
    _, _, _, a23: int, _, _, _, _, _, _ = pressure()
    _, _, _, _, a24: int, _, _, _, _, _ = pressure()
    _, _, _, _, _, a25: int, _, _, _, _ = pressure()
    _, _, _, _, _, _, a26: int, _, _, _ = pressure()
    _, _, _, _, _, _, _, a27: int, _, _ = pressure()
    _, _, _, _, _, _, _, _, a28: int, _ = pressure()
    _, _, _, _, _, _, _, _, _, a29: int = pressure()

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

pressure(): int, int, int, int, int, int, int, int, int, int {
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
    return a0, a1, a2, a3, a4, a5, a6, a7, a8, a9
}
