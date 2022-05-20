main() {
    x: bool = true
    y: bool = false
    z: bool = x == y
    z = x == y
    z = x != z
    z = (x == z) & x;
    z = y | (y != z)
}
