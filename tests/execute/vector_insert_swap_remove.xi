use assert
use io
use conv
use vector

// [random.randint(0, 99) for i in range(100)] + 0 + 99
INPUT: int[] = {11, 41, 93, 51, 47, 53, 37, 1, 89, 92, 57, 11, 15, 17, 24, 96, 98, 27, 23, 83, 19, 81, 22, 17, 45, 33, 2, 51, 97, 61, 70, 67, 22, 23, 10, 17, 5, 63, 31, 67, 15, 99, 72, 46, 68, 18, 38, 62, 21, 97, 29, 71, 27, 0, 83, 88, 12, 84, 55, 4, 72, 63, 30, 57, 92, 74, 67, 30, 16, 32, 26, 18, 46, 21, 86, 46, 43, 54, 56, 37, 22, 37, 7, 20, 75, 82, 23, 81, 11, 52, 20, 89, 90, 72, 45, 49, 37, 0, 60, 52}

main(args: int[][]) {

    output: Vector::<int> = new_vector::<int>()

    i: int = 0
    while i < 100 {
        output.push(i)
        i = i + 1
    }

    j: int = 0
    while j < length(INPUT) {
        k: int = output.swap_remove(INPUT[j])

        assert(k == INPUT[j])
        assert(output.size() == 99)

        output.push(k)
        assert(output.size() == 100)

        output.swap(output.size() - 1, k)

        j = j + 1
    }

    print("100 = ")
    println(unparseInt(output.size()))
}
