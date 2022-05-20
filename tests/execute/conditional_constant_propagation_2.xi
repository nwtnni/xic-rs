use io
use conv

main(args:int[][]) {
    a: int = 0
    b: int = 0
    c: int = a
    d: int = b + 5 * 2

    if c < d {
        print("reachable")
    } else {
        print("unreachable")
    }
}
