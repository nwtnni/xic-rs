use io
use conv

main(args:int[][]) {
    printInt(foo())
}

foo():int {
    return bar(5)[1]
}

bar(x:int):int[] {
    y:int[] = {x,x}
    return y
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