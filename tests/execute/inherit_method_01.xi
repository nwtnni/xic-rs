use io

class A {
  foo() { print("Test") }
}

class B extends A {}

main(args:int[][]) {
  b: B = new B
  b.foo()
}
