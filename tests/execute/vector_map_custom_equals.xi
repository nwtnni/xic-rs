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

class Offset extends Integer {
    value: int
    equals(other: Integer): bool {
        return value + 1 == other.value
    }
}

main(args: int[][]) {
    i: int = 0

    map: VectorMap::<Integer, Integer> = new_vector_map::<Integer, Integer>()

    while i < 100 {
        value: Integer = map.insert(offset(i), integer(i))

        assert(value == null)
        assert(map.contains_key(integer(i + 1)))

        i = i + 1
    }

    print("100 = ")
    println(unparseInt(map.size()))
}

integer(value: int): Integer {
    integer: Integer = new Integer
    integer.value = value
    return integer
}

offset(value: int): Integer {
    offset: Offset = new Offset
    offset.value = value
    return offset
}
