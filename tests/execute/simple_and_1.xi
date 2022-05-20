use io
use conv

main(args:int[][]) {
    printBool(true & true)
}

printBool(b:bool) {
    if b {
        print("True")
    }
    else {
        print("False")
    }
}