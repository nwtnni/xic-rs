class Object {
    integer: int
    value(): int {
        return integer
    }
}

bar(): int {
    return foo::<Object>(new_object(99))
}

new_object(integer: int): Object {
    object: Object = new Object
    object.integer = integer
    return object
}
