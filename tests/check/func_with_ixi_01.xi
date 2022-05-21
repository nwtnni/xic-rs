use A_01

foo() {
    x:A = bar()
    x.foo()
    x.foo1(x)
    x = x.foo2(x)
}
