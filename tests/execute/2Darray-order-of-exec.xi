use io
use conv

main (args:int[][]) {
    arr:int[5][4]

    n:int = 0
    i:int = 0
    while (i < length(arr)) {
        j:int = 0
        while (j < length(arr[i])) {
            arr[i][j] = n
            j = j + 1
            n = n + 1
        }
        i = i + 1
    }

    i = 0
    while (i < length(arr)) {
        j:int = 0
        while (j < length(arr[i])) {
            println(unparseInt(arr[f(i)][f(j)]))
            j = j + 1
        }
        i = i + 1
    }
}

f (i:int) : int {
    print("Index: ")
    println(unparseInt(i))
    return i
}