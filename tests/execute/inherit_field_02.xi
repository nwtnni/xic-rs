use io
use conv

class A {
  a: int
  b: bool
  c: int
}

class B extends A {
  d: int
  e: bool
}

main(args:int[][]) {
  a: A = new A   
  b: B = new B
  
  a.a = 5
  a.c = 10

  b.a = 6
  b.c = 10
  b.d = 6

  print(unparseInt(a.a) + " " + unparseInt(a.c) + "\n")
  print(unparseInt(b.a) + " " + unparseInt(b.c) + " " + unparseInt(b.d))
}
