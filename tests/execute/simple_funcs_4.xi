use io
use conv

main(args:int[][]) {
    print(unparseInt(foo()))
}

foo():int {
    x:int, y:int = bar(5)
    return x+y
}

bar(x:int):int,int {
    return x,1
}