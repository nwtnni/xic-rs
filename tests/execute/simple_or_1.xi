use io
use conv

main(args:int[][]) {
    printBool(true | false)
}

printBool(b:bool) {
    if b {
        print("True")
    }
    else {
        print("False")
    }
}