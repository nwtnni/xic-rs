use io

main(args: int[][]) {
    arr:int[][][] = {{"hello", "world"}, {"foo", "bar", "baz"}}

    arr[0][0] = "no"
    println(arr[0][0])
    arr[0][1][0] = 'W'
    println(arr[0][1])
    arr[1] = {"oops"}
    println(arr[1][0])
    // println(arr[1][1]) // should be caught as out of bounds
}
