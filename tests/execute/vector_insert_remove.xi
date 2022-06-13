use assert
use io
use conv
use vector

// [random.randint(0, 99) for i in range(100)] + 0 + 99
INPUT: int[] = {54, 35, 21, 64, 91, 20, 31, 27, 94, 82, 4, 45, 32, 17, 29, 1, 86, 91, 43, 59, 69, 42, 98, 3, 94, 44, 17, 9, 16, 28, 89, 42, 75, 91, 9, 5, 36, 20, 39, 36, 19, 60, 35, 62, 85, 48, 71, 33, 11, 97, 2, 34, 63, 45, 39, 17, 57, 64, 25, 63, 26, 81, 5, 90, 99, 27, 77, 83, 52, 66, 12, 84, 91, 0, 19, 80, 10, 87, 70, 68, 17, 38, 52, 79, 4, 88, 98, 66, 63, 77, 24, 45, 5, 31, 6, 23, 15, 81, 78, 32}

main(args: int[][]) {

    output: Vector::<int> = newVector::<int>()

    i: int = 0
    while i < 100 {
        output.push(i)
        i = i + 1
    }

    j: int = 0
    while j < length(INPUT) {
        k: int = output.remove(INPUT[j])

        assert(k == INPUT[j])
        assert(output.size() == 99)

        output.insert(k, k)

        assert(output.size() == 100)

        j = j + 1
    }

    print("100 = ")
    println(unparseInt(output.size()))
}
