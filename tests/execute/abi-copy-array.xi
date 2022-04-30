use io
use conv

main(args:int[][]) {
    a:int[5]
    b:int[] = "Hello"

    x:int = 0
    while (x < length(a)) {
        a[x] = b[x]
        printInt(a[x])
        x = x + 1
    }
    println(a)
    println(b)
}

printInt(x:int) {
    println(unparseInt(x))
}
