use io
use conv

main(args:int[][]) {
    i: int = 0
    total: int = 0
    while i < 10 {
        j: int = 0

        while j < 10 {
            k: int = 0

            total = total + (i + i + i)

            while k < 10 {
                total = total + (i + j + i + j) + k
                k = k + 1
            }

            j = j + 1
        }

        i = i + 1
    }

    println("23850 = " + unparseInt(total))
}
