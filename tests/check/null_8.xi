class A {}

class B extends A {}

foo() {
    a: A[] = {new B, new A, null}
}
