use io

main(args: int[][]) {
    arr:int[][][] = {{"hello", "world"}, {"foo", "bar", "baz"}}

    x:int = 0
    i:int = length(arr[0])
    while (x < i) {
        y:int = 0
        j: int = length(arr[x])
        while (y < j) {
            println(arr[x][y])
            y = y + 1
        }
        x = x + 1
    }
}
