use io
use conv

template class A<Functor, T> {
    field: Functor::<T>
}

template class B<T> {
    field: T
    name(): int[] {
        return "B"
    }
}

template class C<T> {
    field: T
    name(): int[] {
        return "C"
    }
}

unparseBool(boolean: bool): int[] {
    if boolean {
        return "True"
    } else {
        return "False"
    }
}

main(args: int[][]) {
    b_int: A::<B, int> = new A::<B, int>
    b_int.field = new B::<int>
    b_int.field.field = 1

    b_bool: A::<B, bool> = new A::<B, bool>
    b_bool.field = new B::<bool>
    b_bool.field.field = true

    c_int: A::<C, int> = new A::<C, int>
    c_int.field = new C::<int>
    c_int.field.field = 2

    c_bool: A::<C, bool> = new A::<C, bool>
    c_bool.field = new C::<bool>
    c_bool.field.field = false

    print("B1 = ")
    print(b_int.field.name())
    println(unparseInt(b_int.field.field))

    print("C2 = ")
    print(c_int.field.name())
    println(unparseInt(c_int.field.field))

    print("BTrue = ")
    print(b_bool.field.name())
    println(unparseBool(b_bool.field.field))

    print("CFalse = ")
    print(c_bool.field.name())
    println(unparseBool(c_bool.field.field))
}
