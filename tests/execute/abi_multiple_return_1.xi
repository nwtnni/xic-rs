use io
use conv

main (args:int[][]) {
    a:int, b:int, c:int = foo()
    printInt(a)
    printInt(b)
    printInt(c)
}

foo():int, int, int{
    return 1,2,3
}

printInt(i:int) {
    println(unparseInt(i))
}