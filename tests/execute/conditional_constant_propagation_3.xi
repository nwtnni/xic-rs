use io
use conv

main(args:int[][]) {
    a: int = 1
    b: int = constant_five()
    c: int = a + 5 * 2

    if constant_false() {
        print("unreachable")
    } else if compare(b, c) {
        print("reachable")
    } else {
        print("unreachable")
    }
}

constant_five(): int {
    return 5
}

constant_false(): bool {
    return false
}

compare(a: int, b: int): bool {
    return a < b
}
