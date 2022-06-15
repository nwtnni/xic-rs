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

class Offset extends Integer {
    value: int
    equals(other: Integer): bool {
        return value + 1 == other.value
    }
}

main(args: int[][]) {
    i: int = 0

    set: VectorSet::<Integer> = new_vector_set::<Integer>()

    while i < 100 {
        assert(set.insert(offset(i)))
        assert(set.contains(integer(i + 1)))
        i = i + 1
    }

    print("100 = ")
    println(unparseInt(set.size()))
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
