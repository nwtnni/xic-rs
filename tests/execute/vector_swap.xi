use assert
use io
use conv
use vector

INPUT: int[] = {22, 74, 13, 12, 2, 28, 99, 26, 7, 99, 29, 48, 35, 13, 81, 73, 81, 49, 13, 10, 55, 50, 99, 50, 23, 25, 18, 59, 87, 79, 28, 60, 39, 35, 77, 5, 33, 94, 65, 0, 18, 22, 3, 72, 11, 92, 64, 31, 8, 97, 63, 28, 66, 36, 17, 4, 20, 39, 67, 30, 33, 3, 91, 26, 64, 52, 95, 99, 78, 98, 36, 9, 49, 41, 84, 97, 78, 91, 18, 56, 39, 35, 10, 28, 81, 16, 99, 39, 70, 34, 10, 66, 65, 24, 71, 8, 73, 19, 67, 68}

main(args: int[][]) {

    output: Vector::<int> = new_vector::<int>()

    i: int = 0
    while i < 100 {
        output.push(i)
        output.swap(i, i)
        assert(output.get(i) == i)
        i = i + 1
    }

    j: int = 0
    while j + 1 < length(INPUT) {
        output.swap(INPUT[j], INPUT[j + 1])
        assert(output.get(INPUT[j]) == INPUT[j + 1])
        assert(output.get(INPUT[j + 1]) == INPUT[j])
        assert(output.size() == 100)
        output.swap(INPUT[j], INPUT[j + 1])
        j = j + 1
    }

    print("100 = ")
    println(unparseInt(output.size()))
}

