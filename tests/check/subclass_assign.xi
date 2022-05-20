class A {}
class B extends A {}
class C extends B {}

main() {
    a: A = new A
    a = new B
    a = new C
}
