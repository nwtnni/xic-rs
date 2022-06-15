use io
use conv
use generic_function_1

main(args: int[][]) {
    i: int = bar()
    j: int = foo::<Object>(newObject(99))

    print(unparseInt(i))
    print(" = ")
    println(unparseInt(j))
}
