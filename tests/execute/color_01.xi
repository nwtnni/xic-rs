use io
use conv

class Color {
  r, g, b: int
  toString(): int[] {
        str:int[] = "Color with r = " + unparseInt(r) + " and g = " + unparseInt(g) + " and b = " + unparseInt(b)
        return str
  }
}

main(args:int[][]) {
  c: Color = new Color
  c.r = 10
  c.g = 20
  c.b = 30
  print(c.toString())
}
