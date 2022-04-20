use io
use conv

main(args:int[][]) {
    printBool(1>=2)
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