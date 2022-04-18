use io
use conv

main(args:int[][]) {
    printInt(length({"hello", "goodbye"}))
}

printInt(x:int) {
    print(unparseInt(x))
}