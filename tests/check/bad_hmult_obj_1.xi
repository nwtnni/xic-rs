foo(x:A, y:A) {
    t:A = x*>>y
}

class A {}
class B1 extends A {}
class B2 extends A {}
class C1 extends B1 {}