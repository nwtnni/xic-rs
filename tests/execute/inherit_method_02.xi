use io

class A {
  foo() { print("A\n") }
}

class B extends A {
  foo() { print("B\n") }
}

main(args:int[][]) {
  a: A = new A
  b: A = new B
  a.foo()
  b.foo()
}
