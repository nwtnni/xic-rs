use io
use conv

class A {
    name(): int[] {
        return "A"
    }
}

class B {
    name(): int[] {
        return "B"
    }
}

class C {
    name(): int[] {
        return "C"
    }
}

template recurse<A, B, C>(a: A, b: B, c: C, depth: int): A, B, C {
    if depth == 0 {
        return a, b, c
    } else {
        b': B, c': C, a': A = recurse::<B, C, A>(b, c, a, depth - 1)
        return a', b', c'
    }
}

main(args: int[][]) {
    c: C, a: A, b: B = recurse::<C, A, B>(new C, new A, new B, 100)

    print("CAB = ")
    print(c.name())
    print(a.name())
    println(b.name())
}
