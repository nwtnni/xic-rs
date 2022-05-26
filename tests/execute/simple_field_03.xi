use io
use conv

class A {
  b: B
}

class B {
  i: int
}

main(args:int[][]) {
  b: B = new B
  b.i = 10
  a: A = new A
  a.b = b
  print(unparseInt(a.b.i))
}
