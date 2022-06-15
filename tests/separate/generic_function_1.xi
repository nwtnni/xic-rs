class Object {
    integer: int
    value(): int {
        return integer
    }
}

bar(): int {
    return foo::<Object>(newObject(99))
}

newObject(integer: int): Object {
    object: Object = new Object
    object.integer = integer
    return object
}
