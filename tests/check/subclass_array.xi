class A {}
class B extends A {}
class C extends B {}
class D extends C {}
class E extends D {}

main() {
    array: A[] = {new E, new D, new B, new C, new A}
}
