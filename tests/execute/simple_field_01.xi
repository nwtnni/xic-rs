use io
use conv

class A {
  x: int

  incr() {
    x = x + 1
  }
}

main(args:int[][]) {
  
  x:int = 0
  a:A = new A

  while x < 10 {
    print(unparseInt(a.x))
    a.incr()
    x = x + 1
  }
}
