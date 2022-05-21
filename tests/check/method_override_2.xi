class A {
    foo1():A {
        return null
    }

    foo2():A {
        return null
    }
}

class B extends A {
    foo1():A {
        return this
    }

    foo2():A {
        return this    
    }
}