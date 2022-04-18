use io
use conv

main (args:int[][]) {
    a:int[] = {2, 3}
    arr:int[f(a, 0)][f(a, 1)][f(a, 0)]
    println(unparseInt(length(arr)))
    println(unparseInt(length(arr[0])))
    println(unparseInt(length(arr[0][0])))
    println(unparseInt(a[0]))
    println(unparseInt(a[1]))
}

f (a: int[], i:int) : int {
    print("Index: ")
    println(unparseInt(i))
    a[i] = a[i] + 1
    return a[i]
}