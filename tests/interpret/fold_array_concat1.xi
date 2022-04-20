use io
use conv

main(args:int[][]) {
    print("Is " + "This " + "Constant Folded?")
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