use io
use conv

main(args:int[][]) {
    i: int = 0
    while i < foo() {
        i = i + 1
    }
}

foo(): int {
    println("foo is only called twice")
    return 1
}
