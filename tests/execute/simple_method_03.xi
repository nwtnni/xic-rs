use io
use conv

class A {
  foo(): A {
    return this
  }

  bar(): int {
    return 1
  }
}

main(args:int[][]) {
  print(unparseInt(new A.foo().bar()))
}
