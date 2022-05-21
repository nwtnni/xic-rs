foo() {
    x:A[]
    x[0] = new A
    x[1] = new B1
    x[2] = new B2
    x[3] = new C1

    y:A[] = x
}

class A {}
class B1 extends A {}
class B2 extends A {}
class C1 extends B1 {}
