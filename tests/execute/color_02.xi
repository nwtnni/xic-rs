use io
use conv

class Point {
  x, y: int

  coords() : int, int { 
    return x, y
  }

  initPoint(x0: int, y0: int): Point {
    x = x0
    y = y0
    return this
  }

  clone(): Point {
    return createPoint(x, y)
  }
}

createPoint(x: int, y: int): Point {
  return new Point.initPoint(x, y)
}

main(args:int[][]) {
  p: Point = createPoint(10, 5)
  x: int, y: int = p.coords()

  c: Point = p.clone()
  a: int, b: int = c.coords()

  print(unparseInt(x) + " " + unparseInt(y) + "\n")
  print(unparseInt(a) + " " + unparseInt(b))
}
