use io
use conv

main(args:int[][]) {
    x:int = 9%5%3
    printInt(x)
}

printInt(x:int) {
    print(unparseInt(x))
}