use io
use conv

main (args:int[][]) {
    x:int = foo()
    a:int, b:int, c:int, d:int, e:int, f:int, g:int, h:int, i:int, j:int, k:int = fun(1,2,3,4,x,bar(),5+2,8,9,10,11)
    println(unparseInt(a))
    println(unparseInt(b))
    println(unparseInt(c))
    println(unparseInt(d))
    println(unparseInt(e))
    println(unparseInt(f))
    println(unparseInt(g))
    println(unparseInt(h))
    println(unparseInt(i))
    println(unparseInt(j))
    println(unparseInt(k))
}

foo() : int {
    return 5
}

bar() : int {
    return 6
}

fun(a:int, b:int, c:int, d:int, e:int, f:int, g:int, h:int, i:int, j:int, k:int):int, int, int, int, int, int, int, int, int, int, int {
    println(unparseInt(a+b+c+d+e+f+g+h+i+j+k))
    return 1,2,3,4,5,6,7,8,9,10,11
}