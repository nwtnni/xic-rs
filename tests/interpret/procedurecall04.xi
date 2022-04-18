use io
use conv

main(args: int[][]) {
    p(47)
}

p(x: int) {
    print("p() called: ")
    println(unparseInt(x))
    if (x <= 0) { return }
    p(x-1)
    print("p() post-call: ")
    println(unparseInt(x))
    return
}