use io
use conv

class A {
  foo(): A {
    return this
  }

  baz(): int {
    return 1
  }
}

main(args:int[][]) {
  print(
    unparseInt(
      new A.foo()
        .foo()
        .foo()
        .foo()
        .foo()
        .foo()
        .foo()
        .baz()
    )
  )
}
