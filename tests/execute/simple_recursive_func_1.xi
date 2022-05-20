use io
use conv

main(args:int[][]) {
    printInt(factorial(5))
}

factorial(n:int):int {
    if n <= 1 {
        return 1
    }
    return n*factorial(n-1)
}

printInt(x:int) {
    print(unparseInt(x))
}