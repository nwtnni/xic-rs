use assert
use io
use conv
use vector_set

class Integer {
    value: int
    equals(other: Integer): bool {
        return value == other.value
    }
}

integer(value: int): Integer {
    integer: Integer = new Integer
    integer.value = value
    return integer
}

main(args: int[][]) {
    i: int = 0

    set: VectorSet::<Integer> = new_vector_set::<Integer>()

    while i < 100 {
        assert(set.insert(integer(i)))
        i = i + 1
        assert(set.size() == i)
    }

    print("100 = ")
    println(unparseInt(set.size()))
}
