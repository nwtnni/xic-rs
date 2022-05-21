foo() {
    x:A[] = {new A, new B1, new B2, new C1}
}

class A {}
class B1 extends A {}
class B2 extends A {}
class C1 extends B1 {}
