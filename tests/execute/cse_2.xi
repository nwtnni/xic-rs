use io
use conv

main(args:int[][]) {
    string: int[] = "hello";

    a: int = string[1]
    b: int = string[2]
    c: int = a + b
    d: int = string[1] + string[2] + (a + b)

    string[1] = d - string[1] - a - b

    print("hlllo = ")
    println(string)
}
