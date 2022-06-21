use io

foo(b: bool): bool {
    return b
}

main(args: int[][]) {

    b: bool = true

    while foo(b) {
        println("Reachable!")
        b = false
    }
}
