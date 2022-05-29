use io
use conv
use cycle_function_2

even(i: int): bool {
    if i == 0 {
        return true
    } else {
        return odd(i - 1)
    }
}

main(args: int[][]) {
    i: int = 0
    while i < 100 {
        print(unparseInt(i))
        if even(i) {
            println(" is even!")
        } else {
            println(" is odd!")
        }
        i = i + 1
    }
}
