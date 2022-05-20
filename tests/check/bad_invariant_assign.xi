class A {}
class B extends A {}
class C extends B {}

main() {
    c: C[] = {new C, new C}
    a: A[] = c
}
