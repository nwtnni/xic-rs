use io
use conv

main(args:int[][]) {
    a:int[foo()]

    x:int = 0
    while (x < length(a)) {
        a[x] = fib(x)
        printInt(a[x])
        x = x + 1
    }
}

foo() : int {
    return 20
}

fib(n:int) : int {
    if (n <= 2) {
        return 1
    }
    return fib(n-1) + fib(n-2)
}

printInt(x:int) {
    println(unparseInt(x))
}
