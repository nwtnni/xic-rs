use io

class A {
    value: bool
}

main(args: int[][]) {
    a: A = new A
    a.value = true

    while a.value {
        println("Reachable!")
        a.value = false
    }
}
