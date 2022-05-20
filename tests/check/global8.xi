x: Empty = empty()
y: Empty = empty().init()

class Empty {
    init(): Empty {
        return this
    }
}

empty(): Empty {
    return new Empty
}
