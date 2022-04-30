use io
use conv

main (args:int[][]) {
    x:int[][][] = {{"Hello1"},{"Hello2"}}
    x[0][0][0] = 'B'
    print(x[0][0])
}