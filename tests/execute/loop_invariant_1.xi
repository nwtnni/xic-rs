use io
use conv

main(args:int[][]) {
    i: int = 0
    a: int = 5
    b: int = 10
    array: int[100]
    total: int = 0

    while (i < length(array)) {
        array[i] = a * b
        i = i + 1
    }

    i = 0

    while (i < length(array)) {
        total = total + array[i]
        i = i + 1
    }

    println("5000 = " + unparseInt(total))
}
