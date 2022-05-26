use io
use conv

class A {
  x: int
}

main(args:int[][]) {
  x: int = 0
  a: A = new A
  a.x = 0
  b: A = new A
  b.x = 0

  while x < 5 {
    a.x = a.x + x 
    b.x = b.x + 1
    x = x + 1
    print(unparseInt(b.x) + " " + unparseInt(a.x) + "\n")
  }
}
