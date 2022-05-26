use io
use conv

class A {
  x: int
}

class B extends A {}

main(args:int[][]) {
  a: A = new A
  x: int = 0
  while x < 5 {
    x = x + 1
    a.x = a.x + 1
    print(unparseInt(a.x) + "\n")
  }
}
