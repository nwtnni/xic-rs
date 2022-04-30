use io
use conv

main(args:int[][]) {
    x:int = 2
    y:int = 3
    if(x>y)
        print(unparseInt(x) + " > " + unparseInt(y) + " = " + "True")
    else
        print(unparseInt(x) + " > " + unparseInt(y) + " = " + "False")
}