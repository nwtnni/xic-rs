use io
use conv

main(args:int[][]) {
    print(unparseInt(foo()))
}

foo():int {
    return bar()
}

bar():int {
    return 1
}