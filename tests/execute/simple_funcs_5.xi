use io
use conv

main(args:int[][]) {
    print(unparseInt(foo()))
}

foo():int {
    x:int, y:int, _ = bar(5)
    return x+y
}

bar(x:int):int,int,int {
    return x,1,0
}