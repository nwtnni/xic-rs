class A {}
class B extends A {}
class C extends B {}

main() {
    a: A[] = foo()
}

foo(): C[] {
    return {new C, new C}
}
