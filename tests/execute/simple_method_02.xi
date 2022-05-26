use io
use conv

class A {
  foo() : int {
    return 1
  }
}

main(args:int[][]) {
  print(unparseInt(new A.foo()))
}
