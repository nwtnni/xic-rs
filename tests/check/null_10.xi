class A {}

class B extends A {}

foo() {
    a: A[] = {null, null, new A, null, new B}
}
