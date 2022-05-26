use io
use conv

class A {

  x: int

  foo(): A {
    x = x + 1
    print("Foo " + unparseInt(x) + "\n")
    return this
  }

  bar(): A {
    x = x + 1
    print("Bar " + unparseInt(x) + "\n")
    return this
  }
}

main(args:int[][]) {
  _ = new A.foo()
    .bar()
    .foo()
    .bar()
    .foo()
}
