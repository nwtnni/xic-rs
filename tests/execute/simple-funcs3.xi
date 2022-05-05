use io
use conv

main(args:int[][]) {
    print(unparseInt(foo()))
}

foo():int {
    return bar(5)
}

bar(x:int):int {
    return x
}