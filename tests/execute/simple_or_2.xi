use io
use conv

main(args:int[][]) {
    printBool(false | false)
}

printBool(b:bool) {
    if b {
        print("True")
    }
    else {
        print("False")
    }
}