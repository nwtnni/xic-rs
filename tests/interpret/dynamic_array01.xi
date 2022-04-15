use io
use conv

main (args:int[][]) {
    n:int = 5
    arr:int[n]

    println(unparseInt(length(arr)))

    i:int = 0
    while (i < length(arr)) {
        arr[i] = i * 2
        println(unparseInt(arr[i]))
        i = i + 1
    }
}