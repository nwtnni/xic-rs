use io
use conv
use shadow_function_1

foo(): bool {
    return false
}

main(args: int[][]) {
    print("5 = ")
    println(unparseInt(bar()))

    print("false = ")
    if foo() {
        println("true")
    } else {
        println("false")
    }
}
