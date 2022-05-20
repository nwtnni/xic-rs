use io
use conv

main(args:int[][]) {
    a: int, b: int = binary(0)
    println("0 is " + unparseInt(a) + unparseInt(b) + " in binary");

    c: int, d: int = binary(1)
    println("1 is " + unparseInt(c) + unparseInt(d) + " in binary");

    e: int, f: int = binary(2)
    println("2 is " + unparseInt(e) + unparseInt(f) + " in binary");

    g: int, h: int = binary(3)
    println("3 is " + unparseInt(g) + unparseInt(h) + " in binary");

    switch: int, len: int = binary(4)
    _, neg1: int = binary(5)
    neg2: int, _ = binary(6)

    println("4 is " + unparseInt(switch))
    println("2 is " + unparseInt(len))
    println("-1 is " + unparseInt(neg1))
    println("-1 is " + unparseInt(neg2))
}

binary(switch: int): int, int {
    output: int[] = {0, 0}

    if switch == 0 {
        output[0] = 0
        output[1] = 0
        return output[1], output[0]
    } else if switch == 1 {
        return 0, 1
    } else if switch == 2 {
        output[1] = 1
        return output[1], output[0]
    } else if switch == 3 {
        output[0] = 1
        output[1] = 1
        return output[1], output[0]
    } else if switch == 4 {
        return switch, length(output)
    } else {
        return -1, -1
    }
}
