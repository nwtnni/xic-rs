use io
use conv

class A {
  b: B
}

class B {
  i: int
  c: C
}

class C {
  a: int
  b: int
}

main(args:int[][]) {
  a: A = new A
  a.b = new B
  a.b.i = 5
  a.b.c = new C
  a.b.c.a = a.b.i + a.b.c.b
  print(unparseInt(5))
}
