use io

class A {
    message(): int[] {
        return "hello"
    }
}

class B {
    message(): int[] {
        return "world"
    }
}

template foo<T>(instance: T): int[] {
    return instance.message()
}

main(args: int[][]) {
    println(foo::<A>(new A))
    println(foo::<B>(new B))
}
