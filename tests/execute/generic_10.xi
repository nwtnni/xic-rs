use io
use conv

main(args: int[][]) {
    object: Object = new Object

    println(object.to_string())

    object = new Container::<Integer>.init(new Integer.init(5))

    println(object.to_string())

    object = new Container::<Boolean>.init(new Boolean.init(true))

    println(object.to_string())
}

class Object {
    to_string(): int[] {
        return "<Object>"
    }
}

template class Container<T> extends Object {
    value: T

    init(value: T): Container::<T> {
        this.value = value
        return this
    }

    to_string(): int[] {
        return "<" + value.to_string() + ">"
    }
}

class Integer {
    value: int

    init(value: int): Integer {
        this.value = value
        return this
    }

    to_string(): int[] {
        return unparseInt(value)
    }
}

class Boolean {
    value: bool

    init(value: bool): Boolean {
        this.value = value
        return this
    }

    to_string(): int[] {
        if value {
            return "true"
        } else {
            return "false"
        }
    }
}
