use io
use conv

main(args:int[][]) {
    printInt(fib(1))
    printInt(fib(2))
    printInt(fib(3))
    printInt(fib(4))
    printInt(fib(5))
    printInt(fib(6))
    printInt(fib(7))
    printInt(fib(8))
    printInt(fib(9))
    printInt(fib(10))
}

// 1 indexed fibonacci
fib(n:int) : int {
    if (n <= 2) {
        return 1
    }
    return fib(n-1) + fib(n-2)
}


printInt(x:int) {
    println(unparseInt(x))
}
