main() {
    x: int[] = {}
    y: int[] = {1, 2, 3}
    z: bool = x == y
    z = x != y

    a: bool[][][] = {{{true}}}
    b: bool[][][] = {{{false}, {true, false}}, {{}}}
    c: bool = a == b
    c = a != b
}
