use io
use conv

main(args:int[][]) {
    x:bool = !false
    printBool(x)
}

printInt(x:int) {
    print(unparseInt(x))
}

printBool(x:bool) {
    if x 
        print("True")
    else
        print("False")
}