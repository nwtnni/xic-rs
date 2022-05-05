use io
use conv

main(args:int[][]) {
    a: int = 0
    b: int = 0
    c: int = 0
    d: int = 0

    i: int = 0

    while (i < 2) {

        j: int = 0

        while (j < 2) {

            k: int = 0

            while (k < 2) {
                println(unparseInt(d))
                d = d + 1
                k = k + 1
            }

            c = c + 1
            j = j + 1
        }

        b = b + 1
        i = i + 1
    }

    println("")
    println("0 = " + unparseInt(a))
    println("2 = " + unparseInt(b))
    println("4 = " + unparseInt(c))
}
