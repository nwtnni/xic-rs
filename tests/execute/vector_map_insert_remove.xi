use assert
use io
use conv
use vector_map

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

    map: VectorMap::<Integer, Integer> = newVectorMap::<Integer, Integer>()

    while i < 100 {
        value: Integer = map.insert(integer(i), integer(i))

        assert(value == null)
        assert(map.remove(integer(i)).value == i)
        assert(map.size() == 0)

        i = i + 1
    }

    print("0 = ")
    println(unparseInt(map.size()))
}
