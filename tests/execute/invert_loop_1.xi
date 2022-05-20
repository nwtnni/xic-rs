use io
use conv

main(args:int[][]) {
    array: int[] = {1, 2, 3, 4, 5}
    i: int = 0

    while i < length(array) {
        println(unparseInt(array[i] + (1 - 1 + 1 - 1)))
        i = i + 1
    }
}
