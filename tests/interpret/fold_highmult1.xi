use io
use conv

main(args:int[][]) {
    x:int = 5*>>1*>>6
    printInt(x)
}

printInt(x:int) {
    print(unparseInt(x))
}