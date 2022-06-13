use io
use conv

template class Container<T> {
    array: T[]

    get(index: int): T {
        return array[index]
    }

    set(index: int, value: T) {
        array[index] = value
    }
}

template create<T>(size: int): Container::<T> {
    container: Container::<T> = new Container::<T>
    array: T[size]
    container.array = array
    return container
}

main(args: int[][]) {
    one: Container::<int> = create::<int>(2)

    i: int = 0
    while i < 2 {
        one.set(i, i)
        println(unparseInt(one.get(i)))
        i = i + 1
    }

    two: Container::<Container::<int>> = create::<Container::<int>>(2)

    println("")

    i = 0

    while i < 2 {

        row: Container::<int> = create::<int>(2)
        two.set(i, row)

        j: int = 0

        while j < 2 {
            two.get(i).set(j, 2 * i + j)
            println(unparseInt(two.get(i).get(j)))
            j = j + 1
        }
        i = i + 1
    }
}
