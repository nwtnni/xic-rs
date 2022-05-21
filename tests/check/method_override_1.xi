class A {
    foo():A {
        return null
    }
}

class B extends A {
    foo():A {
        return this
    }
}