use io
use conv

main(args: int[][]) {
    i: int = 0
    j: int = 0

    while i < 10 {
        j = 0

        while j < 10 {
            print(unparseInt(i))
            print(", ")
            println(unparseInt(j))

            if j > 5 {
                break
            }

            j = j + 1
        }

        i = i + 1
    }
}
